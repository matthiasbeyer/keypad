use std::str::FromStr;

use clap::Parser;
use futures::StreamExt;
use miette::IntoDiagnostic;
use mqtt_format::v5::packets::MqttPacket;
use mqtt_format::v5::packets::publish::MPublish;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

mod action;
mod cli;
mod config;
mod keypad;
mod konst;
mod util;

#[tokio::main]
async fn main() -> miette::Result<()> {
    human_panic::setup_panic!(
        human_panic::Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .authors("Matthias Beyer <mail@beyermatthias.de>")
    );

    let cli = crate::cli::Cli::parse();
    setup_logging(cli.logging.map(From::from));

    tracing::info!("Parsing config now");
    let config = crate::config::Config::load(cli.config_path)
        .await
        .into_diagnostic()?;

    tracing::info!(
        broker = config.mqtt_broker_addr,
        port = config.mqtt_broker_port,
        "Starting MQTT client now"
    );
    let mqtt = cloudmqtt::CloudmqttClient::new(format!(
        "{}:{}",
        config.mqtt_broker_addr, config.mqtt_broker_port
    ))
    .await;

    let event_topic_name = format!(
        "{}/{}",
        config.mqtt_subscribe_prefix,
        crate::konst::KEYPAD_EVENT_TOPIC
    );
    tracing::info!(topic = event_topic_name, "Subscribing event topic now");
    let mut events = mqtt.subscribe(event_topic_name).await;

    let mut key_pad_state = crate::keypad::KeypadState::from_config(&config);
    key_pad_state.publish(&mqtt, &config).await;

    let mut interval = tokio::time::interval(config.interval_duration.unwrap_or(cli.interval));

    let mut key_subscriptions = (0..=24)
        .fold(mqtt.subscription_builder(), |builder, i| {
            let topic = format!("{}/key/{i}", config.mqtt_control_prefix);
            builder.with_subscription(topic)
        })
        .build()
        .await;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::warn!("ctrl-c received, cancelling application.");
                break
            }

            _tick = interval.tick() => {
                tracing::info!("Publishing key state");
                key_pad_state.publish(&mqtt, &config).await
            },

            packet = key_subscriptions.next() => {
                let Some(packet) = packet else {
                    tracing::warn!("control subscription stream seems to have closed");
                    continue
                };

                let MqttPacket::Publish(publish) = packet.get_packet() else {
                    tracing::debug!(?packet, "Ignoring non-publish packet");
                    continue
                };

                tracing::info!(?publish, "Received control packet");
                let Some(target_key) = publish.topic_name.split('/').nth(2) else {
                    tracing::warn!(topic = publish.topic_name, "No target key found in topic name");
                    continue
                };

                let target_key = match target_key.parse::<u8>() {
                    Ok(k) => k,
                    Err(error) => {
                        tracing::warn!(?error, "Could not parse target key as number");
                        continue
                    }
                };
                tracing::debug!(?target_key, "Found target key");

                let control_actions: action::ControlPacket = match serde_json::from_slice(publish.payload) {
                    Ok(a) => a,
                    Err(error) => {
                        tracing::warn!(?error, "Failed to parse control action");
                        continue
                    }
                };

                tracing::info!(n = control_actions.actions.len(), "Received control actions");
                for action in control_actions.actions.into_iter() {
                    tracing::info!(?action, "Applying control action");
                    key_pad_state.run_ctrl_action_on_key(target_key, action);
                }
            },

            next_event = events.next() => {
                if let Some(event) = next_event {
                    tracing::info!("Received event");
                    let MqttPacket::Publish(MPublish { payload, .. }) = event.get_packet() else {
                        tracing::debug!("Ignoring non-publish message");
                        continue;
                    };

                    let num = match std::str::from_utf8(payload).map(f32::from_str) {
                        Ok(Ok(p)) => p,
                        Err(error) => {
                            tracing::warn!(?error, "Failed to parse payload");
                            continue;
                        }
                        Ok(Err(error)) => {
                            tracing::warn!(?error, "Failed to parse payload");
                            continue;
                        }
                    };

                    if num.is_sign_negative() {
                        key_pad_state.released(num.abs() as u8, &mqtt).await;
                    } else {
                        key_pad_state.pressed(num.abs() as u8, &mqtt).await;
                    }
                }
            }
        }
    }

    Ok(())
}

fn setup_logging(log_level: Option<tracing::metadata::Level>) {
    let mut env_filter = tracing_subscriber::EnvFilter::from_default_env();

    if let Some(log_level) = log_level {
        let level_filter = tracing::metadata::LevelFilter::from_level(log_level);
        let directive = tracing_subscriber::filter::Directive::from(level_filter);
        env_filter = env_filter.add_directive(directive);
    }

    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter));

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set global logging subscriber: {:?}", e);
        std::process::exit(1)
    }
}

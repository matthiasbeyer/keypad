#[derive(Clone, Debug)]
pub enum Action {
    ToggleBlinking,
    ToggleBlinkingAlternativeColor,
    PublishMqtt { topic: String, payload: String },
}

impl From<&crate::config::OnPressAction> for Action {
    fn from(value: &crate::config::OnPressAction) -> Self {
        match value {
            crate::config::OnPressAction::ToggleBlinking => Action::ToggleBlinking,
            crate::config::OnPressAction::ToggleBlinkingAlternativeColor => {
                Action::ToggleBlinkingAlternativeColor
            }
            crate::config::OnPressAction::Publish { topic, payload } => Action::PublishMqtt {
                topic: topic.to_string(),
                payload: payload.to_string(),
            },
        }
    }
}

impl From<&crate::config::OnReleaseAction> for Action {
    fn from(value: &crate::config::OnReleaseAction) -> Self {
        match value {
            crate::config::OnReleaseAction::Publish { topic, payload } => Action::PublishMqtt {
                topic: topic.to_string(),
                payload: payload.to_string(),
            },
        }
    }
}

impl Action {
    pub async fn execute(
        &self,
        key_state: &mut crate::keypad::KeyState,
        mqtt_client: &cloudmqtt::CloudmqttClient,
    ) -> Result<(), miette::Error> {
        match self {
            Action::ToggleBlinking => {
                tracing::info!("Action: Toggle blink");
                key_state.toggle_blinking();
                Ok(())
            }

            Action::ToggleBlinkingAlternativeColor => {
                tracing::info!("Action: Toggle blink alternative color");
                key_state.toggle_blinking_alternative_color();
                Ok(())
            }

            Action::PublishMqtt { topic, payload } => {
                tracing::info!(?topic, ?payload, "Action: Publishing");
                mqtt_client.publish(payload, topic).await;
                Ok(())
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ControlPacket {
    pub(crate) actions: Vec<ControlAction>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) enum ControlAction {
    ToggleBlinking,
    ToggleBlinkingAlternativeColor,
}

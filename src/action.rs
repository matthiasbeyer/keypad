#[derive(Clone, Debug)]
pub enum Action {
    ToggleBlinking,
    PublishMqtt { topic: String, payload: String },
}

impl From<&crate::config::OnPressAction> for Action {
    fn from(value: &crate::config::OnPressAction) -> Self {
        match value {
            crate::config::OnPressAction::ToggleBlinking => Action::ToggleBlinking,
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
                key_state.toggle_blinking();
                Ok(())
            }

            Action::PublishMqtt { topic, payload } => {
                mqtt_client.publish(topic, payload).await;
                Ok(())
            }
        }
    }
}

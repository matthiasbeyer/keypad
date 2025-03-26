use cloudmqtt::CloudmqttClient;

use crate::config::PadConfig;

#[derive(Clone, Debug)]
pub struct KeypadState {
    rows: [Row; 5],
}

impl KeypadState {
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            rows: [
                Row([
                    &config.keypad.pad_0_0,
                    &config.keypad.pad_0_1,
                    &config.keypad.pad_0_2,
                    &config.keypad.pad_0_3,
                    &config.keypad.pad_0_4,
                ]
                .into_iter()
                .map(|pad: &PadConfig| KeyState::from(pad))
                .collect()),
                Row([
                    &config.keypad.pad_1_0,
                    &config.keypad.pad_1_1,
                    &config.keypad.pad_1_2,
                    &config.keypad.pad_1_3,
                    &config.keypad.pad_1_4,
                ]
                .into_iter()
                .map(|pad: &PadConfig| KeyState::from(pad))
                .collect()),
                Row([
                    &config.keypad.pad_2_0,
                    &config.keypad.pad_2_1,
                    &config.keypad.pad_2_2,
                    &config.keypad.pad_2_3,
                    &config.keypad.pad_2_4,
                ]
                .into_iter()
                .map(|pad: &PadConfig| KeyState::from(pad))
                .collect()),
                Row([
                    &config.keypad.pad_3_0,
                    &config.keypad.pad_3_1,
                    &config.keypad.pad_3_2,
                    &config.keypad.pad_3_3,
                    &config.keypad.pad_3_4,
                ]
                .into_iter()
                .map(|pad: &PadConfig| KeyState::from(pad))
                .collect()),
                Row([
                    &config.keypad.pad_4_0,
                    &config.keypad.pad_4_1,
                    &config.keypad.pad_4_2,
                    &config.keypad.pad_4_3,
                    &config.keypad.pad_4_4,
                ]
                .into_iter()
                .map(|pad: &PadConfig| KeyState::from(pad))
                .collect()),
            ],
        }
    }

    pub async fn publish(
        &mut self,
        client: &cloudmqtt::CloudmqttClient,
        config: &crate::config::Config,
    ) {
        let mut bytes_pressed: Vec<u8> = Vec::with_capacity((3 * 25) + 3);
        bytes_pressed.extend([0, 0, 0, 25]);

        bytes_pressed.extend(
            self.rows
                .iter_mut()
                .flat_map(|r| r.0.iter_mut())
                .flat_map(|key_state| key_state.color_pressed().as_slice().into_iter()),
        );

        let mut bytes_released: Vec<u8> = Vec::with_capacity((3 * 25) + 3);
        bytes_released.extend([0, 0, 0, 25]);

        bytes_released.extend(
            self.rows
                .iter_mut()
                .flat_map(|r| r.0.iter_mut())
                .flat_map(|key_state| key_state.color_released().as_slice().into_iter()),
        );

        let pressed_pub = client.publish(
            bytes_pressed,
            format!(
                "{}/{}",
                config.mqtt_subscribe_prefix,
                crate::konst::KEYPAD_COLOR_RELEASED_TOPIC
            ),
        );

        let released_pub = client.publish(
            bytes_released,
            format!(
                "{}/{}",
                config.mqtt_subscribe_prefix,
                crate::konst::KEYPAD_COLOR_PRESSED_TOPIC
            ),
        );

        tokio::join!(pressed_pub, released_pub);
    }

    pub async fn pressed(&mut self, index: u8, mqtt: &CloudmqttClient) {
        tracing::debug!(?index, "Pressed");
        match index {
            0..=4 => self.rows[0].pressed(index % 5, mqtt).await,
            5..=9 => self.rows[1].pressed(index % 5, mqtt).await,
            10..=14 => self.rows[2].pressed(index % 5, mqtt).await,
            15..=19 => self.rows[3].pressed(index % 5, mqtt).await,
            20..=24 => self.rows[4].pressed(index % 5, mqtt).await,
            other => tracing::warn!(index = other, "Out of index"),
        }
    }

    pub async fn released(&mut self, index: u8, mqtt: &CloudmqttClient) {
        tracing::debug!(?index, "Released");
        match index {
            0..=4 => self.rows[0].released(index % 5, mqtt).await,
            5..=9 => self.rows[1].released(index % 5, mqtt).await,
            15..=19 => self.rows[3].released(index % 5, mqtt).await,
            10..=14 => self.rows[2].released(index % 5, mqtt).await,
            20..=24 => self.rows[4].released(index % 5, mqtt).await,
            other => tracing::warn!(index = other, "Out of index"),
        }
    }
}

#[derive(Clone, Debug)]
struct Row(Vec<KeyState>);

impl Row {
    pub async fn pressed(&mut self, index: u8, mqtt: &CloudmqttClient) {
        if index < 5 {
            self.0[index as usize].pressed(mqtt).await
        } else {
            tracing::warn!(?index, "Row index out of range");
        }
    }

    pub async fn released(&mut self, index: u8, mqtt: &CloudmqttClient) {
        if index < 5 {
            self.0[index as usize].released(mqtt).await
        } else {
            tracing::warn!(?index, "Row index out of range");
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct KeyState {
    color_pressed: crate::util::Rgb,
    color_released: crate::util::Rgb,
    pressed: bool,
    blinking: bool,
    blink_state: BlinkState,

    on_press: Vec<crate::action::Action>,
    on_release: Vec<crate::action::Action>,
}

impl From<&PadConfig> for KeyState {
    fn from(config: &PadConfig) -> Self {
        Self {
            color_pressed: crate::util::Rgb::from([
                config.pressed[0],
                config.pressed[1],
                config.pressed[2],
            ]),
            color_released: crate::util::Rgb::from([
                config.released[0],
                config.released[1],
                config.released[2],
            ]),
            pressed: false,
            blinking: false,
            blink_state: BlinkState::Off,

            on_press: config
                .on_press
                .iter()
                .map(crate::action::Action::from)
                .collect::<Vec<_>>(),

            on_release: config
                .on_release
                .iter()
                .map(crate::action::Action::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl KeyState {
    async fn pressed(&mut self, mqtt: &CloudmqttClient) {
        self.pressed = true;

        for action in self.on_press.clone().iter() {
            if let Err(error) = action.execute(self, mqtt).await {
                tracing::error!(?error, ?action, "Executing action yielded error");
            }
        }
    }

    async fn released(&mut self, mqtt: &CloudmqttClient) {
        self.pressed = false;

        for action in self.on_release.clone().iter() {
            if let Err(error) = action.execute(self, mqtt).await {
                tracing::error!(?error, ?action, "Executing action yielded error");
            }
        }
    }

    pub(crate) fn toggle_blinking(&mut self) {
        self.blinking = !self.blinking;
    }

    fn color_pressed(&mut self) -> crate::util::Rgb {
        if self.blinking {
            self.color_blinking()
        } else {
            self.color_pressed
        }
    }

    fn color_released(&mut self) -> crate::util::Rgb {
        if self.blinking {
            self.color_blinking()
        } else {
            self.color_released
        }
    }

    fn color_blinking(&mut self) -> crate::util::Rgb {
        match self.blink_state {
            BlinkState::On => {
                self.blink_state = BlinkState::Off;
                self.color_pressed
            }
            BlinkState::Off => {
                self.blink_state = BlinkState::On;
                self.color_released
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum BlinkState {
    On,
    Off,
}

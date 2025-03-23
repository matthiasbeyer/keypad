#[derive(Clone, Debug)]
pub struct KeypadState {
    rows: [Row; 5],
}

impl KeypadState {
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            rows: [
                Row(config
                    .initial_state_released
                    .row_0
                    .iter()
                    .zip(config.initial_state_pressed.row_0.iter())
                    .map(|(rel, pre)| KeyState::new(rel.unpack(), pre.unpack()))
                    .collect()),
                Row(config
                    .initial_state_released
                    .row_1
                    .iter()
                    .zip(config.initial_state_pressed.row_1.iter())
                    .map(|(rel, pre)| KeyState::new(rel.unpack(), pre.unpack()))
                    .collect()),
                Row(config
                    .initial_state_released
                    .row_2
                    .iter()
                    .zip(config.initial_state_pressed.row_2.iter())
                    .map(|(rel, pre)| KeyState::new(rel.unpack(), pre.unpack()))
                    .collect()),
                Row(config
                    .initial_state_released
                    .row_3
                    .iter()
                    .zip(config.initial_state_pressed.row_3.iter())
                    .map(|(rel, pre)| KeyState::new(rel.unpack(), pre.unpack()))
                    .collect()),
                Row(config
                    .initial_state_released
                    .row_4
                    .iter()
                    .zip(config.initial_state_pressed.row_4.iter())
                    .map(|(rel, pre)| KeyState::new(rel.unpack(), pre.unpack()))
                    .collect()),
            ],
        }
    }

    pub async fn publish(
        &self,
        client: &cloudmqtt::CloudmqttClient,
        config: &crate::config::Config,
    ) {
        let mut bytes_pressed: Vec<u8> = Vec::with_capacity((3 * 25) + 3);
        bytes_pressed.extend([0, 0, 0, 25]);

        bytes_pressed.extend(
            self.rows
                .to_vec()
                .into_iter()
                .flat_map(|r| r.0.into_iter())
                .flat_map(|key_state| key_state.color_pressed.as_slice().into_iter()),
        );

        let mut bytes_released: Vec<u8> = Vec::with_capacity((3 * 25) + 3);
        bytes_released.extend([0, 0, 0, 25]);

        bytes_released.extend(
            self.rows
                .to_vec()
                .into_iter()
                .flat_map(|r| r.0.into_iter())
                .flat_map(|key_state| key_state.color_released.as_slice().into_iter()),
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

    pub fn pressed(&mut self, index: u8) {
        tracing::debug!(?index, "Pressed");
        match index {
            0..=4 => self.rows[0].pressed(index % 5),
            5..=9 => self.rows[1].pressed(index % 5),
            10..=14 => self.rows[2].pressed(index % 5),
            15..=19 => self.rows[3].pressed(index % 5),
            20..=24 => self.rows[4].pressed(index % 5),
            other => tracing::warn!(index = other, "Out of index"),
        }
    }

    pub fn released(&mut self, index: u8) {
        tracing::debug!(?index, "Released");
        match index {
            0..=4 => self.rows[0].released(index % 5),
            5..=9 => self.rows[1].released(index % 5),
            10..=14 => self.rows[2].released(index % 5),
            15..=19 => self.rows[3].released(index % 5),
            20..=24 => self.rows[4].released(index % 5),
            other => tracing::warn!(index = other, "Out of index"),
        }
    }
}

#[derive(Clone, Debug)]
struct Row(Vec<KeyState>);

impl Row {
    pub fn pressed(&mut self, index: u8) {
        if index < 5 {
            self.0[index as usize].pressed()
        } else {
            tracing::warn!(?index, "Row index out of range");
        }
    }

    pub fn released(&mut self, index: u8) {
        if index < 5 {
            self.0[index as usize].released()
        } else {
            tracing::warn!(?index, "Row index out of range");
        }
    }
}

#[derive(Clone, Debug)]
struct KeyState {
    color_pressed: crate::util::Rgb,
    color_released: crate::util::Rgb,
    pressed: bool,
}

impl KeyState {
    fn new(color_released: [u8; 3], color_pressed: [u8; 3]) -> Self {
        Self {
            color_pressed: crate::util::Rgb::from(color_pressed),
            color_released: crate::util::Rgb::from(color_released),
            pressed: false,
        }
    }

    fn pressed(&mut self) {
        self.pressed = true;
    }

    fn released(&mut self) {
        self.pressed = false;
    }
}

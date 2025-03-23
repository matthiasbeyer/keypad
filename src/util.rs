#[derive(Clone, Copy, Debug)]
pub struct Rgb([u8; 3]);

impl From<[u8; 3]> for Rgb {
    fn from(value: [u8; 3]) -> Self {
        Self(value)
    }
}

impl Rgb {
    pub fn as_slice(&self) -> [u8; 3] {
        self.0
    }
}

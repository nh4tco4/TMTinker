mod tmcore;

pub use tmcore::TMCore;

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug, Copy)]
pub struct NonZeroU32(u32);

impl NonZeroU32 {
    pub fn new(number: u32) -> Self {
        let value = if number < 1 { 1 } else { number };
        Self(value)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

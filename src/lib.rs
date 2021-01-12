use num::clamp;
use serde::{Deserialize, Serialize};
use std::num::{NonZeroU64, ParseIntError};
use std::time::Duration;
use std::{fmt, str::FromStr};
//use rppal::i2c::I2c;

mod mailbox;
pub use mailbox::*;
mod config;
pub use config::*;

pub const VCIO_DEV: &str = "/dev/vcio";
pub const I2C_BUS: u8 = 1;
pub const I2C_FAN_CTRLR_ADDR: u16 = 0x1A;
pub const CONFIG_SYS_PATH: &str = "/etc/argonone/config.toml";

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct I2cBus(pub u8);

impl Default for I2cBus {
    fn default() -> Self {
        I2cBus(I2C_BUS)
    }
}

impl fmt::Display for I2cBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for I2cBus {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().parse::<u8>()?;
        Ok(I2cBus(num))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct I2cAddress(pub u16);

impl Default for I2cAddress {
    fn default() -> Self {
        I2cAddress(I2C_FAN_CTRLR_ADDR)
    }
}

impl fmt::Display for I2cAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

impl FromStr for I2cAddress {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let num = if s.starts_with("0x") {
            u16::from_str_radix(s.trim_start_matches("0x"), 16)?
        } else {
            s.parse::<u16>()?
        };
        Ok(I2cAddress(num))
    }
}

#[derive(Debug, Clone, err_derive::Error)]
pub enum ParseFanSpeedError {
    #[error(display = "Failed to parse fan speed {}", _0)]
    ParseIntError(#[error(from)] ParseIntError),

    #[error(display = "Invalid fan speed {}, valid values are 0..=100", _0)]
    Invalid(u8),
}

/// Fan speed, as a percentage, valid values are 0..=100
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct FanSpeed(u8);

impl FanSpeed {
    pub const MAX: Self = FanSpeed(100);
}

impl From<FanSpeed> for u8 {
    fn from(fs: FanSpeed) -> Self {
        fs.0
    }
}

impl Default for FanSpeed {
    fn default() -> Self {
        FanSpeed(25)
    }
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl FromStr for FanSpeed {
    type Err = ParseFanSpeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().parse::<u8>()?;
        if num > Self::MAX.0 {
            Err(ParseFanSpeedError::Invalid(num))
        } else {
            Ok(FanSpeed(num))
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct DegreesC(pub u8);

impl DegreesC {
    // TODO - redo this
    pub fn from_f32(t: f32) -> Self {
        let int_t = clamp(t, u8::MIN as f32, u8::MAX as f32) as u8;
        DegreesC(int_t)
    }
}

impl From<u8> for DegreesC {
    fn from(t: u8) -> Self {
        DegreesC(t)
    }
}

impl fmt::Display for DegreesC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} C", self.0)
    }
}

impl FromStr for DegreesC {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().parse::<u8>()?;
        Ok(DegreesC(num))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct UpdateIntervalSeconds(pub NonZeroU64);

impl From<NonZeroU64> for UpdateIntervalSeconds {
    fn from(sec: NonZeroU64) -> Self {
        UpdateIntervalSeconds(sec)
    }
}

impl From<UpdateIntervalSeconds> for Duration {
    fn from(i: UpdateIntervalSeconds) -> Self {
        Duration::from_secs(i.0.get())
    }
}

impl fmt::Display for UpdateIntervalSeconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.0.get())
    }
}

#[derive(Debug, Clone, err_derive::Error)]
pub enum ParseUpdateIntervalSecondsError {
    #[error(display = "Failed to parse update interval, {}", _0)]
    ParseError(#[error(from)] ParseIntError),

    #[error(display = "Failed to parse update interval, must be non-zero seconds (u64)")]
    Invalid,
}

impl FromStr for UpdateIntervalSeconds {
    type Err = ParseUpdateIntervalSecondsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sec = s.trim().parse::<u64>()?;
        Ok(NonZeroU64::new(sec)
            .ok_or(ParseUpdateIntervalSecondsError::Invalid)?
            .into())
    }
}

// TODO - move this to a fn somewhere, not part of Config
//
// newtype for fan speed/etc
// proptest, min < max, never exceeds max, clamps, etc
//
// defaults
// tempC=fan-percent
// 55=10
// 60=55
// 65=100
//
// clamp the f32 to min/max as a u8
//
// basic linear interpolate, or https://rosettacode.org/wiki/Map_range#Rust
//pub(crate) fn interpolate_fan_speed(&self, temperature: u8) -> FanSpeed {
//    todo!()
//}

#[cfg(test)]
mod test {
    use super::*;

    // TODO
}

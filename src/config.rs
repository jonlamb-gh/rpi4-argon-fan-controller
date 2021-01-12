use crate::{DegreesC, FanSpeed, UpdateIntervalSeconds};
use log::info;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug, err_derive::Error)]
pub enum ConfigLoadError {
    #[error(display = "Failed to configuration file {:?}, {}", _0, _1)]
    Io(PathBuf, io::Error),

    #[error(display = "Configuration file {:?} is invalid, {}", _0, _1)]
    Invalid(PathBuf, toml::de::Error),

    #[error(display = "{}", _0)]
    Check(#[error(from)] ConfigCheckError),
}

#[derive(Debug, err_derive::Error)]
pub enum ConfigCheckError {
    #[error(display = "The configuration file temperature range is invalid")]
    InvalidTemperatureRange,

    #[error(display = "The configuration file fan speed range is invalid")]
    InvalidFanSpeedRange,

    #[error(display = "The configuration file fan speed min is invalid")]
    InvalidFanSpeedMin,

    #[error(display = "The configuration file fan speed max is invalid")]
    InvalidFanSpeedMax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Time interval to check temperature and update fan speed
    pub update_interval_seconds: UpdateIntervalSeconds,
    /// Min temp, degrees C
    pub temperature_min: DegreesC,
    /// Max temp, degrees C
    pub temperature_max: DegreesC,
    /// Min fan speed percentage
    pub fan_speed_min: FanSpeed,
    /// Max fan speed percentage
    pub fan_speed_max: FanSpeed,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            update_interval_seconds: UpdateIntervalSeconds(NonZeroU64::new(30).unwrap()),
            temperature_min: 55.into(),
            temperature_max: 65.into(),
            fan_speed_min: FanSpeed(10),
            fan_speed_max: FanSpeed::MAX,
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigLoadError> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigLoadError::Io(path.as_ref().to_path_buf(), e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| ConfigLoadError::Invalid(path.as_ref().to_path_buf(), e))?;
        config.check()?;
        info!("Loaded configuration file {}", path.as_ref().display());
        info!("Update interval {}", config.update_interval_seconds);
        info!(
            "Temperature range {}..={} C",
            u8::from(config.temperature_min),
            u8::from(config.temperature_max)
        );
        info!(
            "Fan speed range {}..={} %",
            u8::from(config.fan_speed_min),
            u8::from(config.fan_speed_max)
        );
        Ok(config)
    }

    pub fn check(&self) -> Result<(), ConfigCheckError> {
        if self.temperature_min.0 >= self.temperature_max.0 {
            Err(ConfigCheckError::InvalidTemperatureRange)
        } else if self.fan_speed_min.0 >= self.fan_speed_max.0 {
            Err(ConfigCheckError::InvalidFanSpeedRange)
        } else if self.fan_speed_min.0 > FanSpeed::MAX.0 {
            Err(ConfigCheckError::InvalidFanSpeedMin)
        } else if self.fan_speed_max.0 > FanSpeed::MAX.0 {
            Err(ConfigCheckError::InvalidFanSpeedMax)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sane_default() {
        // TODO
    }
}

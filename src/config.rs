use crate::{DegreesC, FanSpeed, UpdateIntervalSeconds};
use log::info;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
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

#[derive(Debug, PartialEq, err_derive::Error)]
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
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
            update_interval_seconds: UpdateIntervalSeconds(NonZeroU32::new(30).unwrap()),
            temperature_min: 33.into(),
            temperature_max: 65.into(),
            fan_speed_min: FanSpeed(0),
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
pub(crate) mod test {
    use super::*;
    use crate::test::*;
    use proptest::prelude::*;
    use std::cmp::Ordering;

    prop_compose! {
        pub(crate) fn gen_config()(
            i in gen_update_interval_seconds(),
            t_a in gen_degrees_c(),
            t_b in gen_degrees_c(),
            fs_a in gen_fan_speed(),
            fs_b in gen_fan_speed(),
        ) -> Config {
            let (t_min, t_max) = match t_a.cmp(&t_b) {
                Ordering::Less => (t_a, t_b),
                Ordering::Greater => (t_b, t_a),
                Ordering::Equal => (t_a.0.saturating_sub(1).into(), t_b.0.saturating_add(1).into()),
            };
            let (fs_min, fs_max) = match fs_a.cmp(&fs_b) {
                Ordering::Less => (fs_a, fs_b),
                Ordering::Greater => (fs_b, fs_a),
                Ordering::Equal => {
                    (
                        FanSpeed::new_unchecked(fs_a.0.saturating_sub(1)),
                        FanSpeed::new_unchecked(std::cmp::min(FanSpeed::MAX.0, fs_b.0.saturating_add(1)))
                    )
                }
            };
            assert!(t_max > t_min);
            assert!(fs_max > fs_min);
            let config = Config {
                update_interval_seconds: i,
                temperature_min: t_min,
                temperature_max: t_max,
                fan_speed_min: fs_min,
                fan_speed_max: fs_max,
            };
            assert!(config.check().is_ok());
            config
        }
    }

    proptest! {
        #[test]
        fn round_trip(config in gen_config()) {
            let out_dir = tempfile::tempdir().unwrap();
            let out_path = out_dir.path().join("config.toml");
            fs::write(
                &out_path,
                toml::to_string_pretty(&config).unwrap().as_bytes(),
            )
            .unwrap();
            let in_config = Config::load(&out_path).unwrap();
            prop_assert!(in_config.check().is_ok());
            prop_assert_eq!(config, in_config);
        }
    }

    #[test]
    fn sane_default() {
        assert_eq!(
            Config::default(),
            Config {
                update_interval_seconds: UpdateIntervalSeconds(NonZeroU32::new(30).unwrap()),
                temperature_min: 33.into(),
                temperature_max: 65.into(),
                fan_speed_min: FanSpeed::new(0).unwrap(),
                fan_speed_max: FanSpeed::MAX,
            }
        );
    }

    #[test]
    fn config_check_errors() {
        let c = Config {
            update_interval_seconds: UpdateIntervalSeconds(NonZeroU32::new(30).unwrap()),
            temperature_min: 1.into(),
            temperature_max: 0.into(),
            fan_speed_min: FanSpeed::new(10).unwrap(),
            fan_speed_max: FanSpeed::MAX,
        };
        assert_eq!(c.check(), Err(ConfigCheckError::InvalidTemperatureRange));
        let c = Config {
            update_interval_seconds: UpdateIntervalSeconds(NonZeroU32::new(30).unwrap()),
            temperature_min: 0.into(),
            temperature_max: 1.into(),
            fan_speed_min: FanSpeed::new(10).unwrap(),
            fan_speed_max: FanSpeed::new(1).unwrap(),
        };
        assert_eq!(c.check(), Err(ConfigCheckError::InvalidFanSpeedRange));
    }
}

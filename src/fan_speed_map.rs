use crate::{DegreesC, FanSpeed};
use num::clamp;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanSpeedMap {
    temperature_min: DegreesC,
    temperature_max: DegreesC,
    fan_speed_min: FanSpeed,
    fan_speed_max: FanSpeed,
    map: HashMap<DegreesC, FanSpeed>,
}

impl FanSpeedMap {
    pub fn new(
        temperature_min: DegreesC,
        temperature_max: DegreesC,
        fan_speed_min: FanSpeed,
        fan_speed_max: FanSpeed,
    ) -> Self {
        let t_min = u8::from(temperature_min);
        let t_max = u8::from(temperature_max);
        let s_min = u8::from(fan_speed_min);
        let s_max = u8::from(fan_speed_max);
        // TODO - make an error type or use Config ref
        assert!(t_max > t_min, "Invalid temperature range");
        assert!(s_max > s_min, "Invalid fan speed range");

        let mut map = HashMap::new();
        for t in t_min..=t_max {
            let s_f64 = map_range((t_min as _, t_max as _), (s_min as _, s_max as _), t as _);
            let s = FanSpeed::new_unchecked(clamp(s_f64 as _, s_min, s_max));
            let t = DegreesC::from(t);
            log::debug!("{} -> {}", t, s);
            map.insert(t, s);
        }

        FanSpeedMap {
            temperature_min,
            temperature_max,
            fan_speed_min,
            fan_speed_max,
            map,
        }
    }

    pub fn get(&self, temp: DegreesC) -> FanSpeed {
        if temp < self.temperature_min {
            self.fan_speed_min
        } else if temp > self.temperature_max {
            self.fan_speed_max
        } else {
            *self.map.get(&temp).unwrap_or_else(|| {
                panic!(
                    "Internal map corrupt? No value for key {}\n{:?}",
                    temp, self.map
                )
            })
        }
    }
}

// https://rosettacode.org/wiki/Map_range#Rust
fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::gen_config;
    use crate::test::gen_degrees_c;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_mappings(config in gen_config(), temp in gen_degrees_c()) {
            let map = FanSpeedMap::new(
                config.temperature_min,
                config.temperature_max,
                config.fan_speed_min,
                config.fan_speed_max,
            );
            let fs = map.get(temp);
            prop_assert!(fs >= config.fan_speed_min);
            prop_assert!(fs <= config.fan_speed_max);
        }
    }
}

use chrono::prelude::*;
use log::info;
use rpi_mailbox::{
    error::Error, firmware_revision, get_board_model, get_board_revision, get_temperature, Mailbox,
};
use rppal::i2c::I2c;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

const VCIO_DEV: &str = "/dev/vcio";
const I2C_BUS: u8 = 1;
const I2C_FAN_CTRLR_ADDR: u16 = 0x1A;
const CONFIG_SYS_PATH: &str = "/etc/argonone/config.toml";

// TODOs
// - structopt, with manual set-fan... subcommand/cmd
// - error types/handling
// - sanity check config values
// - fan speed, u8, percent 0..=100
// - refactor the crate into lib+cli, lib stuff can test on the build host

// defaults
// tempC=fan-percent
// 55=10
// 60=55
// 65=100
//
// toml config in /etc, with default, cli opts optional path
// basic linear interpolate, or https://rosettacode.org/wiki/Map_range#Rust

// https://github.com/kounch/argonone/blob/feature/RaspberryPi4/argononed.py
// https://docs.golemparts.com/rppal/0.11.2/rppal/i2c/struct.I2c.html
// https://github.com/golemparts/rppal/blob/master/examples/i2c_ds3231.rs

#[derive(Debug, StructOpt)]
#[structopt(name = "argon-fan-ctl", about = "Argon ONE M.2 Fan Controller")]
pub struct Opts {
    // TODO - newtypes with Default, FromStr impls
    /// I2C bus
    #[structopt(long, default_value = "1")]
    pub i2c_bus: u8,

    /// Fan controller I2C address
    #[structopt(long, default_value = "0x1A")]
    pub i2c_addr: u16,

    /// VideoCore IO device path
    #[structopt(long, name = "vcio device path", default_value = VCIO_DEV)]
    pub vcio: PathBuf,

    /// VideoCore temperature sensor ID
    #[structopt(long, name = "sensor id", default_value = "0")]
    pub temperature_sensor_id: u32,

    /// Configuration file path
    #[structopt(long, short = "c", default_value = CONFIG_SYS_PATH)]
    pub config: PathBuf,

    /// Write the default configuration file to path and exit
    #[structopt(long, name = "path")]
    pub write_default_config: Option<PathBuf>,

    /// Set the fan speed (percentage) and exit
    #[structopt(long, name = "percentage")]
    pub set_fan_speed: Option<u8>,
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let opts = Opts::from_args();

    // let decoded: Config = toml::from_str(toml_str).unwrap();
    // let content = &fs::read_to_string(path)?;
    // let config = toml::from_str(content)?;

    let mb = Mailbox::new(VCIO_DEV)?;

    let rev = firmware_revision(&mb)?;
    let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(rev as i64, 0), Utc);
    info!("Firmware revision: {}", date.format("%b %e %Y %T"));

    let model = get_board_model(&mb)?;
    info!("Board model: 0x{:08x}", model);

    let rev = get_board_revision(&mb)?;
    info!("Board revision: 0x{:08x}", rev);

    let temperature = get_temperature(&mb, 0)?;
    info!("temperature: {} C", temperature as f32 / 1000.0);

    let mut i2c = I2c::with_bus(I2C_BUS).unwrap();

    //i2c.set_slave_address(I2C_FAN_CTRLR_ADDR).unwrap();
    //i2c.smbus_send_byte(50).unwrap();

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Time interval to check temperature and update fan speed
    pub interval: String,
    /// Min temp, degrees C
    pub temperature_min: u8,
    /// Max temp, degrees C
    pub temperature_max: u8,
    /// Min fan speed percentage
    pub fan_percentage_min: u8,
    /// Max fan speed percentage
    pub fan_percentage_max: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval: "30s".to_string(),
            temperature_min: 55,
            temperature_max: 65,
            fan_percentage_min: 10,
            fan_percentage_max: 100,
        }
    }
}

impl Config {
    // newtype for fan speed/etc
    // proptest, min < max, never exceeds max, clamps, etc
    pub(crate) fn interpolate_fan_speed(&self, temperature: u8) -> u8 {
        todo!()
    }
}

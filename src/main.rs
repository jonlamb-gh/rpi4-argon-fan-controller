use lib::*;
use log::info;
use std::{fs, path::PathBuf};
use structopt::StructOpt;
//use rppal::i2c::I2c;

// TODOs
// - lib tests (run on the build host)
// - wire up the error handling path to output via error!
// - sanity check config values/ranges
// - rm the pub's in the newtypes, To/From's
// - turn on lint checks

// https://github.com/kounch/argonone/blob/feature/RaspberryPi4/argononed.py
// https://docs.golemparts.com/rppal/0.11.2/rppal/i2c/struct.I2c.html
// https://github.com/golemparts/rppal/blob/master/examples/i2c_ds3231.rs

#[derive(Debug, StructOpt)]
#[structopt(name = "argon-fan-ctl", about = "Argon ONE M.2 Fan Controller")]
pub struct Opts {
    /// I2C bus
    #[structopt(long, default_value)]
    pub i2c_bus: I2cBus,

    /// Fan controller I2C address
    #[structopt(long, default_value)]
    pub i2c_addr: I2cAddress,

    /// VideoCore IO device path
    #[structopt(long, name = "vcio device path", default_value = VCIO_DEV)]
    pub vcio: PathBuf,

    /// Configuration file path
    #[structopt(long, short = "c", default_value = CONFIG_SYS_PATH)]
    pub config: PathBuf,

    /// Write the default configuration file to path and exit
    #[structopt(long, name = "path")]
    pub write_default_config: Option<PathBuf>,

    /// Set the fan speed (percentage, 0..=100) and exit
    #[structopt(long, name = "percentage")]
    pub set_fan_speed: Option<FanSpeed>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let opts = Opts::from_args();

    if let Some(path) = &opts.write_default_config {
        let config = Config::default();
        fs::write(path, toml::to_string_pretty(&config)?.as_bytes())?;
        info!("Wrote default configuration file to {}", path.display());
        return Ok(());
    }

    let config = Config::load(&opts.config)?;

    //let mut mb = Mailbox::new(&opts.vcio)?;

    //let temperature = mb.temperature()?;
    //info!("temperature: {} C", temperature);

    //let mut i2c = I2c::with_bus(I2C_BUS).unwrap();
    //i2c.set_slave_address(I2C_FAN_CTRLR_ADDR).unwrap();
    //i2c.smbus_send_byte(50).unwrap();

    Ok(())
}

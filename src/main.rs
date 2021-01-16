use lib::*;
use log::{debug, error, info, warn};
use rppal::i2c::I2c;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::{
    fs,
    path::PathBuf,
    process, thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;

// TODOs
// - rm the pub's in the newtypes, To/From's
// - turn on lint checks
// - consider setting default min fan speed to 0/off

// ex RUST_LOG=lib,argon_fan_ctl=debug /tmp/argon-fan-ctl -c /tmp/config.toml
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
    #[structopt(long, name = "percentage", conflicts_with = "get_fan_speed")]
    pub set_fan_speed: Option<FanSpeed>,

    /// Print the temperature and exit
    #[structopt(long, conflicts_with = "percentage")]
    pub get_temp: bool,
}

fn main() {
    match do_main() {
        Ok(()) => (),
        Err(e) => {
            error!("{}", e);
            process::exit(exitcode::SOFTWARE);
        }
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let opts = Opts::from_args();

    if let Some(fan_speed) = opts.set_fan_speed {
        let mut i2c = I2c::with_bus(opts.i2c_bus.into())?;
        i2c.set_slave_address(opts.i2c_addr.into())?;
        i2c.smbus_send_byte(fan_speed.into())?;
        debug!("Set the fan speed to {}", fan_speed);
        return Ok(());
    }

    if opts.get_temp {
        let mut mb = Mailbox::new(&opts.vcio)?;
        let temp_c = mb.temperature()?;
        println!("Temperature: {}", temp_c);
        return Ok(());
    }

    if let Some(path) = &opts.write_default_config {
        let config = Config::default();
        fs::write(path, toml::to_string_pretty(&config)?.as_bytes())?;
        info!("Wrote default configuration file to {}", path.display());
        return Ok(());
    }

    let config = Config::load(&opts.config)?;

    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            info!("Shutting down");
        } else {
            warn!("Forcing exit");
            process::exit(exitcode::SOFTWARE);
        }
    })?;

    let mut i2c = I2c::with_bus(opts.i2c_bus.into())?;
    i2c.set_slave_address(opts.i2c_addr.into())?;
    let mut mb = Mailbox::new(&opts.vcio)?;
    let map = FanSpeedMap::new(
        config.temperature_min,
        config.temperature_max,
        config.fan_speed_min,
        config.fan_speed_max,
    );

    let fan_speed = FanSpeed::default();
    debug!("Setting default fan speed {}", fan_speed);
    i2c.smbus_send_byte(fan_speed.into())?;

    let mut sched = Scheduler::new(Instant::now(), config.update_interval_seconds.into());
    while running.load(Ordering::SeqCst) == 0 {
        if sched.update(Instant::now()) {
            let temp_c = DegreesC::from_f32(mb.temperature()?);
            let fan_speed = map.get(temp_c);
            i2c.smbus_send_byte(fan_speed.into())?;
            debug!("Temp {}, fan speed {}", temp_c, fan_speed);
        }

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

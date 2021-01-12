use chrono::prelude::*;
use log::info;
use rpi_mailbox::{firmware_revision, get_board_model, get_board_revision, get_temperature};
use std::path::Path;

#[derive(Debug, Clone, err_derive::Error)]
#[error(display = "{}", _0)]
pub struct MailboxError(rpi_mailbox::error::ErrorKind);

impl From<rpi_mailbox::error::Error> for MailboxError {
    fn from(e: rpi_mailbox::error::Error) -> Self {
        MailboxError(e.kind().clone())
    }
}

pub struct Mailbox(rpi_mailbox::Mailbox);

impl Mailbox {
    const SOC_SENSOR_ID: u32 = 0;

    pub fn new<P: AsRef<Path>>(vcio_dev: P) -> Result<Self, MailboxError> {
        let mb = rpi_mailbox::Mailbox::new(vcio_dev.as_ref())?;

        let rev = firmware_revision(&mb)?;
        let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(rev as i64, 0), Utc);
        info!("Firmware revision: {}", date.format("%b %e %Y %T"));

        let model = get_board_model(&mb)?;
        info!("Board model: 0x{:08x}", model);

        let rev = get_board_revision(&mb)?;
        info!("Board revision: 0x{:08x}", rev);

        Ok(Mailbox(mb))
    }

    /// Returns the temperature in degrees C
    pub fn temperature(&mut self) -> Result<f32, MailboxError> {
        let raw = get_temperature(&self.0, Self::SOC_SENSOR_ID)?;
        Ok(raw as f32 / 1000.0)
    }
}

//! Peripherals

use crate::serial::SerialPort;
use core::mem::replace;

/// A structure that holds references to peripherals
pub struct Peripherals {
    pub serial : Option<SerialPort>,
}

impl Peripherals {
    /// Lock the serial port
    pub fn lock_serial(&mut self) -> SerialPort {
        let p = replace(&mut self.serial, None);
        p.unwrap()
    }

    /// Unlock the serial port
    pub fn release_serial(&mut self, serial : SerialPort) {
        let _ = replace(&mut self.serial, Some(serial));
    }
}

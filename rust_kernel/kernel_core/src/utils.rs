/// Support for `print!()` macro using SerialPort
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        unsafe {
            // Lock the serial port, print data and release it
            let mut serial = PERIPHERALS.lock_serial();
            let _ = core::fmt::Write::write_fmt(&mut serial, 
                                                format_args!($($arg)*));
            PERIPHERALS.release_serial(serial);
        }
    }
}

/// Support for `println!()` macro using SerialPort
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

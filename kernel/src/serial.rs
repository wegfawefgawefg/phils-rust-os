use core::fmt::Result;
use core::fmt::Write;
use uart_16550::SerialPort;

pub struct SerialWriter {
    pub serial_port: SerialPort,
}

impl SerialWriter {
    pub fn new() -> SerialWriter {
        let mut serial_port: SerialPort = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        SerialWriter { serial_port }
    }
}

impl Default for SerialWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            self.serial_port.send(byte);
        }
        Ok(())
    }
}

#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use uart_16550::SerialPort;

use serial::SerialWriter;
use vga_text_mode::VGAWriter;

mod serial;
mod vga_text_mode;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut vga = VGAWriter::new();
    vga.col = 0; // Reset to start of screen
    write!(&mut vga, "PANIC: {}", info).unwrap();
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

fn delay() {
    for _ in 0..10000 {
        // Busy-wait; do nothing
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vga = VGAWriter::new();
    let mut ser = SerialWriter::new();

    for i in 0..1000 {
        write!(&mut vga, "{}\n", i).unwrap(); // Use '\n' to insert new lines
        delay();
    }

    for i in 0..25 {
        write!(&mut ser, "{} ", i).unwrap(); // Use '\n' to insert new lines
        delay();
    }

    exit_qemu(QemuExitCode::Failed);

    loop {}
}

// need a function for doing write! macro over the serial port

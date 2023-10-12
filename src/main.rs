#![no_std]
#![no_main]

use core::panic::PanicInfo;
// use uart_16550::SerialPort;
// use serial::SerialWriter;

use crate::{
    vga_text_mode::VGAColorCode,
    vga_text_mode_drawing::{draw_circle, draw_line},
    vga_text_mode_terminal::VGA_TEXT_MODE_TERMINAL,
};

pub mod init;
// pub mod interupts;
pub mod serial;
pub mod vga_text_mode;
pub mod vga_text_mode_drawing;
pub mod vga_text_mode_terminal;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    let mut vga_terminal = VGA_TEXT_MODE_TERMINAL.lock();
    vga_terminal.col = 0; // Reset to start of screen
    let _ = write!(&mut *vga_terminal, "PANIC: {}", info);

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
    println!("Hello World{}", "!");

    // Define some color codes

    let block = 0xDB;

    // Draw circles
    draw_circle(40, 12, 10, block, VGAColorCode::Red);
    draw_circle(60, 12, 8, block, VGAColorCode::Green);
    draw_circle(80, 12, 6, block, VGAColorCode::Blue);

    // Draw lines
    draw_line(5, 5, 20, 20, block, VGAColorCode::Cyan);
    draw_line(20, 5, 5, 20, block, VGAColorCode::Cyan);

    loop {}
}

// need a function for doing write! macro over the serial port

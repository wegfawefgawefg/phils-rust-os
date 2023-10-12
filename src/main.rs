#![no_std]
#![no_main]

use core::panic::PanicInfo;
// use uart_16550::SerialPort;
// use serial::SerialWriter;

use crate::{
    random::{pseudo_rand, pseudo_rand_in_range_u32},
    vga_text_mode::{VGAColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, VGA_TEXT_MODE},
    vga_text_mode_drawing::{draw_circle, draw_line},
    vga_text_mode_terminal::VGA_TEXT_MODE_TERMINAL,
};

pub mod init;
// pub mod interupts;
pub mod random;
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

    let mut x = BUFFER_WIDTH as i32 / 2;
    let mut y = BUFFER_HEIGHT as i32 / 2;
    let r = 3; // Radius
    let mut dx = 2; // Velocity in x direction
    let mut dy = 2; // Velocity in y direction

    loop {
        // Clear screen
        VGA_TEXT_MODE.lock().clear_screen();

        // Draw circle
        draw_circle(
            x as usize,
            y as usize,
            r as usize,
            block,
            VGAColorCode::White,
        );

        // Update position based on velocity
        x += dx;
        y += dy;

        // Collision detection and velocity update
        if x - r <= 0 || x + r >= BUFFER_WIDTH as i32 {
            dx = -dx;
        }
        if y - r <= 0 || y + r >= BUFFER_HEIGHT as i32 {
            dy = -dy;
        }

        // Delay for visibility
        for _ in 0..60 {
            delay();
        }
    }

    loop {}
}

// need a function for doing write! macro over the serial port

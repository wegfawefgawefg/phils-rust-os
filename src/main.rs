#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{fmt::Write, sync::atomic::Ordering};

use core::panic::PanicInfo;
// use uart_16550::SerialPort;
// use serial::SerialWriter;

use interupts::PICS;
use util::halt_loop;
use vga_text_mode_terminal::CURSOR_TOGGLE_FLAG;

use crate::{
    interupts::{overflow_stack, trigger_page_fault},
    util::delay,
    vga_text_mode::{VGAColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, VGA_TEXT_MODE},
    vga_text_mode_drawing::{draw_circle, draw_line, draw_point, BLOCK},
    vga_text_mode_terminal::VGA_TEXT_MODE_TERMINAL,
};

pub mod gdt;
pub mod interupts;
pub mod random;
pub mod serial;
pub mod util;
pub mod vga_text_mode;
pub mod vga_text_mode_drawing;
pub mod vga_text_mode_terminal;

pub fn init() {
    gdt::init();
    interupts::init();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();

    // loop {
    //     print!("Hello World!");
    //     delay();
    //     delay();
    // }
    halt_loop();
}

// need a function for doing write! macro over the serial port
pub fn draw_bouncy_ball() {
    // Draw circles
    draw_circle(40, 12, 10, BLOCK, VGAColorCode::Red);
    draw_circle(60, 12, 8, BLOCK, VGAColorCode::Green);
    draw_circle(80, 12, 6, BLOCK, VGAColorCode::Blue);

    // Draw lines
    draw_line(5, 5, 20, 20, BLOCK, VGAColorCode::Cyan);
    draw_line(20, 5, 5, 20, BLOCK, VGAColorCode::Cyan);

    let mut x = BUFFER_WIDTH as i32 / 2;
    let mut y = BUFFER_HEIGHT as i32 / 2;
    let r = 3; // Radius
    let mut dx = 2; // Velocity in x direction
    let mut dy = 2; // Velocity in y direction

    loop {
        // Clear screen
        VGA_TEXT_MODE.lock().clear_screen();

        // // Draw circle
        draw_circle(
            x as usize,
            y as usize,
            r as usize,
            BLOCK,
            VGAColorCode::DarkGray,
        );

        // draw a white spec in the topish right of the circle
        draw_point((x + 2) as usize, (y - 1) as usize, VGAColorCode::White);

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

        println!("x: {}, y: {}", x, y);
        VGA_TEXT_MODE_TERMINAL.lock().row = 0;

        // // Delay for visibility
        for _ in 0..60 {
            delay();
        }
    }
}

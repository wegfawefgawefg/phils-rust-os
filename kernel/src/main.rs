#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::fmt::Write;

use bootloader_api::{entry_point, BootInfo};

use framebuffer::run_bouncy_circles;
use interupts::PICS;
use serial::SerialWriter;
use util::halt_loop;

pub mod framebuffer;
pub mod gdt;
pub mod interupts;
pub mod random;
pub mod serial;
pub mod util;
pub mod vga_text_mode;
pub mod vga_text_mode_drawing;
pub mod vga_text_mode_terminal;

entry_point!(kernel_main);

pub fn init() {
    gdt::init();
    interupts::init();
    unsafe { PICS.lock().initialize() };
    interupts::init_pit(60);
    x86_64::instructions::interrupts::enable();
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = SerialWriter::new();
    let _ = writeln!(&mut serial, "kernel: start");

    init();
    let _ = writeln!(&mut serial, "kernel: init done");

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let _ = writeln!(
            &mut serial,
            "kernel: framebuffer {}x{} {:?}",
            info.width,
            info.height,
            info.pixel_format
        );
        run_bouncy_circles(framebuffer);
    }

    let _ = writeln!(&mut serial, "kernel: no framebuffer");
    halt_loop();
}

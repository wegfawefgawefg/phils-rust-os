use core::fmt::Write;

use core::panic::PanicInfo;

use crate::vga_text_mode_terminal::VGA_TEXT_MODE_TERMINAL;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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

pub fn delay() {
    for _ in 0..10000 {
        // Busy-wait; do nothing
    }
}

pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

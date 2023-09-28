#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut vga = VGABuffer::new();
    vga.col = 0; // Reset to start of screen
    write!(&mut vga, "PANIC: {}", info).unwrap();
    loop {}
}

const BUFFER_HEIGHT: isize = 25;
const BUFFER_WIDTH: isize = 80;

struct VGABuffer {
    buffer: *mut u8,
    col: isize,
    row: isize,
}

impl VGABuffer {
    fn new() -> VGABuffer {
        VGABuffer {
            buffer: 0xb8000 as *mut u8,
            col: 0,
            row: 0,
        }
    }

    fn write_byte(&mut self, byte: u8, color: u8) {
        if self.col >= BUFFER_WIDTH {
            self.new_line();
        }

        let pos = (self.row * BUFFER_WIDTH + self.col) as isize;
        unsafe {
            *self.buffer.offset(pos * 2) = byte;
            *self.buffer.offset(pos * 2 + 1) = color;
        }

        self.col += 1;
    }

    fn write_str(&mut self, s: &str, color: u8) {
        for byte in s.bytes() {
            self.write_byte(byte, color);
        }
    }

    fn shift_rows_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character =
                    unsafe { *self.buffer.offset((row * BUFFER_WIDTH + col) as isize * 2) };
                let color = unsafe {
                    *self
                        .buffer
                        .offset((row * BUFFER_WIDTH + col) as isize * 2 + 1)
                };
                unsafe {
                    *self
                        .buffer
                        .offset(((row - 1) * BUFFER_WIDTH + col) as isize * 2) = character;
                    *self
                        .buffer
                        .offset(((row - 1) * BUFFER_WIDTH + col) as isize * 2 + 1) = color;
                }
            }
        }
    }

    fn clear_row(&mut self, row: isize) {
        for col in 0..BUFFER_WIDTH {
            unsafe {
                *self.buffer.offset((row * BUFFER_WIDTH + col) as isize * 2) = 0;
                *self
                    .buffer
                    .offset((row * BUFFER_WIDTH + col) as isize * 2 + 1) = 0;
            }
        }
    }

    fn new_line(&mut self) {
        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
            self.col = 0;
        } else {
            self.shift_rows_up();
            self.clear_row(BUFFER_HEIGHT - 1);
            self.col = 0;
        }
    }

    fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

impl Write for VGABuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            match byte {
                // ASCII newline
                0x0a => {
                    self.new_line();
                }
                // Regular byte
                _ => {
                    self.write_byte(byte, 0xb);
                }
            }
        }
        Ok(())
    }
}

macro_rules! println {
    ($($arg:tt)*) => ({
        let mut vga = VGABuffer::new();
        write!(&mut vga, $($arg)*).unwrap();
    });
}

fn delay() {
    for _ in 0..10000 {
        // Busy-wait; do nothing
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vga = VGABuffer::new();

    for i in 0.. {
        write!(&mut vga, "{}\n", i).unwrap(); // Use '\n' to insert new lines
        delay();
    }

    loop {}
}

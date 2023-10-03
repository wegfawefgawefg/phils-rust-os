use core::fmt::Write;

pub const BUFFER_HEIGHT: isize = 25;
pub const BUFFER_WIDTH: isize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]

pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

pub struct VGAWriter {
    pub buffer: *mut u8,
    pub col: isize,
    pub row: isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorData(u8);

impl ColorData {
    fn new(foreground: Color, background: Color, blink: bool) -> ColorData {
        ColorData(((blink as u8) << 7) | ((background as u8) << 4) | (foreground as u8))
    }
}

impl VGAWriter {
    pub fn new() -> VGAWriter {
        VGAWriter {
            buffer: 0xb8000 as *mut u8,
            col: 0,
            row: 0,
        }
    }

    pub fn write_byte(&mut self, byte: u8, color: Color) {
        if self.col >= BUFFER_WIDTH {
            self.new_line();
        }

        let pos = (self.row * BUFFER_WIDTH + self.col) as isize;
        unsafe {
            *self.buffer.offset(pos * 2) = byte;
            *self.buffer.offset(pos * 2 + 1) = color as u8;
        }

        self.col += 1;
    }

    pub fn write_str(&mut self, s: &str, color: Color) {
        for byte in s.bytes() {
            self.write_byte(byte, color);
        }
    }

    pub fn shift_rows_up(&mut self) {
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

    pub fn clear_row(&mut self, row: isize) {
        for col in 0..BUFFER_WIDTH {
            unsafe {
                *self.buffer.offset((row * BUFFER_WIDTH + col) as isize * 2) = 0;
                *self
                    .buffer
                    .offset((row * BUFFER_WIDTH + col) as isize * 2 + 1) = 0;
            }
        }
    }

    pub fn new_line(&mut self) {
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

impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            match byte {
                // ASCII newline
                0x0a => {
                    self.new_line();
                }
                // Regular byte
                _ => {
                    self.write_byte(byte, Color::White);
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

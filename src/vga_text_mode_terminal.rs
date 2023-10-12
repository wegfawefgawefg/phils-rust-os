use core::fmt::Write;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::vga_text_mode::{VGAColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, VGA_TEXT_MODE};

lazy_static! {
    pub static ref VGA_TEXT_MODE_TERMINAL: Mutex<VGATextModeTerminal> =
        Mutex::new(VGATextModeTerminal::new());
}

pub struct VGATextModeTerminal {
    pub col: usize,
    pub row: usize,
}

impl VGATextModeTerminal {
    pub fn new() -> VGATextModeTerminal {
        VGATextModeTerminal { col: 0, row: 0 }
    }

    pub fn write_byte(&mut self, byte: u8, color: VGAColorCode) {
        if self.col >= BUFFER_WIDTH {
            self.new_line();
        }

        VGA_TEXT_MODE.lock().write(byte, color, self.col, self.row);

        self.col += 1;
    }

    pub fn write_str(&mut self, s: &str, color: VGAColorCode) {
        for byte in s.bytes() {
            self.write_byte(byte, color);
        }
    }

    pub fn shift_rows_up(&mut self) {
        VGA_TEXT_MODE.lock().shift_up_by_x(1);
    }

    pub fn clear_row(&mut self, row: usize) {
        VGA_TEXT_MODE.lock().clear_row(row);
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

    pub fn clear_screen(&mut self) {
        VGA_TEXT_MODE.lock().clear_screen();

        self.col = 0;
        self.row = 0;
    }
}

impl Default for VGATextModeTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for VGATextModeTerminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            match byte {
                // ASCII newline
                0x0a => {
                    self.new_line();
                }
                // Regular byte
                _ => {
                    self.write_byte(byte, VGAColorCode::White);
                }
            }
        }
        Ok(())
    }
}

// macro_rules! println {
//     ($($arg:tt)*) => ({
//         let mut vga = VGABuffer::new();
//         write!(&mut vga, $($arg)*).unwrap();
//     });
// }

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!(&mut *VGA_TEXT_MODE_TERMINAL.lock(), $($arg)*);
    });
}

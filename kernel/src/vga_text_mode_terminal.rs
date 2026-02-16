use core::fmt::Write;

use crate::vga_text_mode::{VGAColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, VGA_TEXT_MODE};
use core::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref VGA_TEXT_MODE_TERMINAL: Mutex<VGATextModeTerminal> =
        Mutex::new(VGATextModeTerminal::new());
}

pub static CURSOR_TOGGLE_FLAG: AtomicBool = AtomicBool::new(false);

pub struct VGATextModeTerminal {
    pub col: usize,
    pub row: usize,
}

impl VGATextModeTerminal {
    pub fn new() -> VGATextModeTerminal {
        VGATextModeTerminal { col: 0, row: 0 }
    }

    pub fn toggle_cursor(&mut self) {
        let cursor_visible = CURSOR_TOGGLE_FLAG.load(Ordering::SeqCst);

        // If cursor is to be shown, write an underscore or block at the current position
        let character = if cursor_visible { b'_' } else { b' ' };

        VGA_TEXT_MODE
            .lock()
            .write(character, VGAColorCode::White, self.col, self.row);
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
            match byte {
                // ASCII newline
                0x0a | 0x0d => {
                    self.new_line();
                }
                // backspace
                0x08 => {
                    VGA_TEXT_MODE.lock().write(b' ', color, self.col, self.row);
                    if self.col > 0 {
                        self.col -= 1;
                    }
                }
                // Regular byte
                _ => {
                    self.write_byte(byte, color);
                }
            }
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
        self.write_str(s, VGAColorCode::White);
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use x86_64::instructions::interrupts::without_interrupts;
        use $crate::vga_text_mode_terminal::VGA_TEXT_MODE_TERMINAL;
        without_interrupts(|| {
            let mut terminal = VGA_TEXT_MODE_TERMINAL.lock();
            let _ = write!(&mut *terminal, $($arg)*);
            let _ = write!(&mut *terminal, "\n");
        });
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use x86_64::instructions::interrupts::without_interrupts;
        use $crate::vga_text_mode_terminal::VGA_TEXT_MODE_TERMINAL;
        without_interrupts(|| {
            let mut terminal = VGA_TEXT_MODE_TERMINAL.lock();
            let _ = write!(&mut *terminal, $($arg)*);
        });
    });
}

use lazy_static::lazy_static;
use volatile::Volatile; //  prevents the compiler from optimizing away reads and writes to memory that has side-effects.

use spin::Mutex;

lazy_static! {
    pub static ref VGA_TEXT_MODE: Mutex<VGATextMode> = Mutex::new(VGATextMode::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum VGAColorCode {
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

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct VGAChar {
    pub char: u8,
    pub color_code: VGAColorCode,
}

#[repr(transparent)]
struct Buffer {
    /**  tldr: Volatile prevents the compiler from optimizing away reads and writes to memory that has side-effects.
     * There may be times when only the VGA rendering hardware reads the buffer after writes,
     * in which case the compiler will optimize away the writes entirely.
     * Volatile prevents this optimization from happening.
     */
    chars: [[Volatile<VGAChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct VGATextMode {
    buffer: &'static mut Buffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorData(u8);

impl ColorData {
    fn new(foreground: VGAColorCode, background: VGAColorCode, blink: bool) -> ColorData {
        ColorData(((blink as u8) << 7) | ((background as u8) << 4) | (foreground as u8))
    }
}

impl VGATextMode {
    pub fn new() -> VGATextMode {
        VGATextMode {
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }

    pub fn write(&mut self, byte: u8, color: VGAColorCode, col: usize, row: usize) {
        self.buffer.chars[row][col].write(VGAChar {
            char: byte,
            color_code: color,
        });
    }

    pub fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.write(b' ', VGAColorCode::Black, col, row);
        }
    }

    pub fn clear_column(&mut self, col: usize) {
        for row in 0..BUFFER_HEIGHT {
            self.write(b' ', VGAColorCode::Black, col, row);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }

    ///////////////////////////////////////////////////////
    /// SHIFTING WITHOUT WRAP
    ///////////////////////////////////////////////////////

    /// Shifts the screen to the left by x columns, dropping the leftmost x columns. The rightmost x columns are left blank.
    pub fn shift_left_by_x(&mut self, x: usize) {
        for row in 0..BUFFER_HEIGHT {
            for col in x..BUFFER_WIDTH {
                self.buffer.chars[row][col - x] = self.buffer.chars[row][col].clone();
            }
            self.clear_subcolumn(BUFFER_WIDTH - x, BUFFER_WIDTH, row);
        }
    }

    /// Shifts the screen to the right by x columns, dropping the rightmost x columns. The leftmost x columns are left blank.
    pub fn shift_right_by_x(&mut self, x: usize) {
        for row in 0..BUFFER_HEIGHT {
            for col in (0..BUFFER_WIDTH - x).rev() {
                self.buffer.chars[row][col + x] = self.buffer.chars[row][col].clone();
            }
            self.clear_subcolumn(0, x, row);
        }
    }

    /// Shifts all rows up by x, dropping the top x rows. The bottom x rows are left blank.
    pub fn shift_up_by_x(&mut self, x: usize) {
        for row in x..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row - x][col] = self.buffer.chars[row][col].clone();
            }
        }
        self.clear_row_range(BUFFER_HEIGHT - x, BUFFER_HEIGHT);
    }

    /// Shifts all rows down by x, dropping the bottom x rows. The top x rows are left blank.
    pub fn shift_down_by_x(&mut self, x: usize) {
        for row in (x..BUFFER_HEIGHT).rev() {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col] = self.buffer.chars[row - x][col].clone();
            }
        }
        self.clear_row_range(0, x);
    }

    ///////////////////////////////////////////////////////
    /// SHIFTING WITH WRAP
    ///////////////////////////////////////////////////////

    /// Shifts the screen to the left by x columns, wrapping the leftmost x columns to the right.
    pub fn shift_left_by_x_with_wrap(&mut self, x: usize) {
        let mut tmp_row = [VGAChar {
            char: b' ',
            color_code: VGAColorCode::Black,
        }; BUFFER_WIDTH];

        for row in 0..BUFFER_HEIGHT {
            // Read the volatile data into tmp_row
            for col in 0..x {
                tmp_row[col] = self.buffer.chars[row][col].read();
            }

            // Shift the columns to the left
            for col in x..BUFFER_WIDTH {
                self.buffer.chars[row][col - x].write(self.buffer.chars[row][col].read());
            }

            // Wrap around the shifted-out columns
            for col in BUFFER_WIDTH - x..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(tmp_row[col + x - BUFFER_WIDTH]);
            }
        }
    }

    /// Shifts the screen to the right by x columns, wrapping the rightmost x columns to the left.
    pub fn shift_right_by_x_with_wrap(&mut self, x: usize) {
        let mut tmp_row = [VGAChar {
            char: b' ',
            color_code: VGAColorCode::Black,
        }; BUFFER_WIDTH];

        for row in 0..BUFFER_HEIGHT {
            for col in BUFFER_WIDTH - x..BUFFER_WIDTH {
                tmp_row[col + x - BUFFER_WIDTH] = self.buffer.chars[row][col].read();
            }
            for col in (0..BUFFER_WIDTH - x).rev() {
                self.buffer.chars[row][col + x].write(self.buffer.chars[row][col].read());
            }
            for col in 0..x {
                self.buffer.chars[row][col].write(tmp_row[col]);
            }
        }
    }

    /// Shifts all rows up by x, wrapping the top x rows to the bottom.
    pub fn shift_up_by_x_with_wrap(&mut self, x: usize) {
        let mut tmp_rows = [[VGAChar {
            char: b' ',
            color_code: VGAColorCode::Black,
        }; BUFFER_WIDTH]; BUFFER_HEIGHT];

        for row in 0..x {
            for col in 0..BUFFER_WIDTH {
                tmp_rows[row][col] = self.buffer.chars[row][col].read();
            }
        }

        for row in x..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row - x][col].write(self.buffer.chars[row][col].read());
            }
        }

        for row in BUFFER_HEIGHT - x..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(tmp_rows[row + x - BUFFER_HEIGHT][col]);
            }
        }
    }

    /// Shifts all rows down by x, wrapping the bottom x rows to the top.
    pub fn shift_down_by_x_with_wrap(&mut self, x: usize) {
        let mut tmp_rows = [[VGAChar {
            char: b' ',
            color_code: VGAColorCode::Black,
        }; BUFFER_WIDTH]; BUFFER_HEIGHT];

        for row in BUFFER_HEIGHT - x..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                tmp_rows[row + x - BUFFER_HEIGHT][col] = self.buffer.chars[row][col].read();
            }
        }

        for row in (x..BUFFER_HEIGHT).rev() {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(self.buffer.chars[row - x][col].read());
            }
        }

        for row in 0..x {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(tmp_rows[row][col]);
            }
        }
    }

    ///////////////////////////////////////////////////////
    /// UTILS
    ///////////////////////////////////////////////////////

    /**  Clears a range of full rows. */
    fn clear_row_range(&mut self, start_row: usize, end_row: usize) {
        for row in start_row..end_row {
            self.clear_row(row);
        }
    }

    /** Clears a range of full columns across all rows. */
    fn clear_column_range(&mut self, start_col: usize, end_col: usize) {
        for row in 0..BUFFER_HEIGHT {
            for col in start_col..end_col {
                self.buffer.chars[row][col].write(VGAChar {
                    char: b' ',
                    color_code: VGAColorCode::Black,
                });
            }
        }
    }

    /** Clears a section of a single row. */
    fn clear_subrow(&mut self, start_col: usize, end_col: usize, row: usize) {
        for col in start_col..end_col {
            self.buffer.chars[row][col].write(VGAChar {
                char: b' ',
                color_code: VGAColorCode::Black,
            });
        }
    }

    /**  Clears a section of a single column. */
    fn clear_subcolumn(&mut self, start_row: usize, end_row: usize, col: usize) {
        for row in start_row..end_row {
            self.buffer.chars[row][col].write(VGAChar {
                char: b' ',
                color_code: VGAColorCode::Black,
            });
        }
    }

    /**  Clears a rectangular area. */
    fn clear_rect(&mut self, start_row: usize, end_row: usize, start_col: usize, end_col: usize) {
        for row in start_row..end_row {
            for col in start_col..end_col {
                self.buffer.chars[row][col].write(VGAChar {
                    char: b' ',
                    color_code: VGAColorCode::Black,
                });
            }
        }
    }
}

impl Default for VGATextMode {
    fn default() -> Self {
        Self::new()
    }
}

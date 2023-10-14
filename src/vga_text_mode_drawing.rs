use crate::vga_text_mode::{VGAColorCode, VGATextMode, BUFFER_HEIGHT, BUFFER_WIDTH, VGA_TEXT_MODE};

pub const BLOCK: u8 = 0xDB;

pub fn draw_line(x1: usize, y1: usize, x2: usize, y2: usize, byte: u8, color: VGAColorCode) {
    let mut vga = VGA_TEXT_MODE.lock();
    let mut x = x1 as isize;
    let mut y = y1 as isize;

    let dx = if x1 > x2 {
        x1 as isize - x2 as isize
    } else {
        x2 as isize - x1 as isize
    };
    let dy = if y1 > y2 {
        y1 as isize - y2 as isize
    } else {
        y2 as isize - y1 as isize
    };

    let dx1 = dx << 1;
    let dy1 = dy << 1;

    let if_cond_x = if x1 > x2 { -1 } else { 1 };
    let if_cond_y = if y1 > y2 { -1 } else { 1 };

    if dy <= dx {
        let mut mut_y = dy1 - dx; // dy*2 - dx
        for _ in x1..=x2 {
            vga.write(byte, color, x as usize, y as usize);
            if mut_y >= 0 {
                y += if_cond_y;
                mut_y -= dx1;
            }
            mut_y += dy1;
            x += if_cond_x;
        }
    } else {
        let mut mut_x = dx1 - dy; // dx*2 - dy
        for _ in y1..=y2 {
            vga.write(byte, color, x as usize, y as usize);
            if mut_x >= 0 {
                x += if_cond_x;
                mut_x -= dy1;
            }
            mut_x += dx1;
            y += if_cond_y;
        }
    }
}

pub fn draw_circle(cx: usize, cy: usize, r: usize, byte: u8, color: VGAColorCode) {
    let mut vga = VGA_TEXT_MODE.lock();

    // Nested function to safely write to the VGA buffer
    let safe_write = |vga: &mut VGATextMode, x: isize, y: isize| {
        if x >= 0 && y >= 0 && x < BUFFER_WIDTH as isize && y < BUFFER_HEIGHT as isize {
            vga.write(byte, color, x as usize, y as usize);
        }
    };

    let mut x = r as isize;
    let mut y = 0isize;
    let mut p = 1 - r as isize;

    safe_write(&mut vga, cx as isize + x, cy as isize - y);
    safe_write(&mut vga, cx as isize - x, cy as isize + y);
    safe_write(&mut vga, cx as isize + x, cy as isize + y);
    safe_write(&mut vga, cx as isize - x, cy as isize - y);

    safe_write(&mut vga, cx as isize + y, cy as isize - x);
    safe_write(&mut vga, cx as isize - y, cy as isize + x);
    safe_write(&mut vga, cx as isize + y, cy as isize + x);
    safe_write(&mut vga, cx as isize - y, cy as isize - x);

    while x > y {
        y += 1;

        if p <= 0 {
            p = p + 2 * y + 1;
        } else {
            x -= 1;
            p = p + 2 * y - 2 * x + 1;
        }

        if x < y {
            break;
        }

        safe_write(&mut vga, cx as isize + x, cy as isize - y);
        safe_write(&mut vga, cx as isize - x, cy as isize + y);
        safe_write(&mut vga, cx as isize + x, cy as isize + y);
        safe_write(&mut vga, cx as isize - x, cy as isize - y);

        if x != y {
            safe_write(&mut vga, cx as isize + y, cy as isize - x);
            safe_write(&mut vga, cx as isize - y, cy as isize + x);
            safe_write(&mut vga, cx as isize + y, cy as isize + x);
            safe_write(&mut vga, cx as isize - y, cy as isize - x);
        }
    }
}

pub fn draw_point(x: usize, y: usize, color: VGAColorCode) {
    if x >= BUFFER_WIDTH || y >= BUFFER_HEIGHT {
        return;
    }

    let mut vga = VGA_TEXT_MODE.lock();
    vga.write(BLOCK, color, x, y);
}

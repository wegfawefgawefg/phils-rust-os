use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

use crate::interupts;

const MAX_BACKBUFFER_BYTES: usize = 1280 * 720 * 4;
static mut BACKBUFFER: [u8; MAX_BACKBUFFER_BYTES] = [0; MAX_BACKBUFFER_BYTES];

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

struct Renderer<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
}

impl<'a> Renderer<'a> {
    fn new(buffer: &'a mut [u8], info: FrameBufferInfo) -> Self {
        Self { buffer, info }
    }

    fn width(&self) -> i32 {
        self.info.width as i32
    }

    fn height(&self) -> i32 {
        self.info.height as i32
    }

    fn clear(&mut self, color: Color) {
        let bpp = self.info.bytes_per_pixel;
        match self.info.pixel_format {
            PixelFormat::Rgb => {
                for px in self.buffer.chunks_exact_mut(bpp) {
                    px[0] = color.r;
                    if bpp > 1 {
                        px[1] = color.g;
                    }
                    if bpp > 2 {
                        px[2] = color.b;
                    }
                }
            }
            PixelFormat::Bgr => {
                for px in self.buffer.chunks_exact_mut(bpp) {
                    px[0] = color.b;
                    if bpp > 1 {
                        px[1] = color.g;
                    }
                    if bpp > 2 {
                        px[2] = color.r;
                    }
                }
            }
            PixelFormat::U8 => {
                let gray = ((u16::from(color.r) + u16::from(color.g) + u16::from(color.b)) / 3) as u8;
                for px in self.buffer.chunks_exact_mut(bpp) {
                    px[0] = gray;
                }
            }
            PixelFormat::Unknown { .. } => {
                for px in self.buffer.chunks_exact_mut(bpp) {
                    px[0] = color.r;
                    if bpp > 1 {
                        px[1] = color.g;
                    }
                    if bpp > 2 {
                        px[2] = color.b;
                    }
                }
            }
            _ => {
                for px in self.buffer.chunks_exact_mut(bpp) {
                    px[0] = color.r;
                }
            }
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width() || y >= self.height() {
            return;
        }

        let x = x as usize;
        let y = y as usize;
        let pixel_index = y * self.info.stride + x;
        let byte_index = pixel_index * self.info.bytes_per_pixel;
        if byte_index + self.info.bytes_per_pixel > self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[byte_index] = color.r;
                self.buffer[byte_index + 1] = color.g;
                self.buffer[byte_index + 2] = color.b;
            }
            PixelFormat::Bgr => {
                self.buffer[byte_index] = color.b;
                self.buffer[byte_index + 1] = color.g;
                self.buffer[byte_index + 2] = color.r;
            }
            PixelFormat::U8 => {
                let gray = ((u16::from(color.r) + u16::from(color.g) + u16::from(color.b)) / 3) as u8;
                self.buffer[byte_index] = gray;
            }
            PixelFormat::Unknown { .. } => {
                self.buffer[byte_index] = color.r;
                if self.info.bytes_per_pixel > 1 {
                    self.buffer[byte_index + 1] = color.g;
                }
                if self.info.bytes_per_pixel > 2 {
                    self.buffer[byte_index + 2] = color.b;
                }
            }
            _ => {}
        }
    }

    fn draw_filled_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= r2 {
                    self.set_pixel(cx + x, cy + y, color);
                }
            }
        }
    }

    fn present(&self, target: &mut [u8]) {
        target.copy_from_slice(self.buffer);
    }
}

#[derive(Clone, Copy)]
struct Ball {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    radius: i32,
    color: Color,
}

impl Ball {
    fn step(&mut self, width: i32, height: i32) {
        self.x += self.vx;
        self.y += self.vy;

        if self.x - self.radius <= 0 || self.x + self.radius >= width - 1 {
            self.vx = -self.vx;
        }
        if self.y - self.radius <= 0 || self.y + self.radius >= height - 1 {
            self.vy = -self.vy;
        }
    }
}

pub fn run_bouncy_circles(framebuffer: &mut FrameBuffer) -> ! {
    let info = framebuffer.info();
    let framebuffer_bytes = framebuffer.buffer_mut();
    let byte_len = info.byte_len;
    if byte_len == 0 || byte_len > MAX_BACKBUFFER_BYTES {
        loop {
            core::hint::spin_loop();
        }
    }

    let backbuffer_ptr = core::ptr::addr_of_mut!(BACKBUFFER) as *mut u8;
    let backbuffer = unsafe { core::slice::from_raw_parts_mut(backbuffer_ptr, byte_len) };
    let mut renderer = Renderer::new(backbuffer, info);

    let mut balls = [
        Ball {
            x: renderer.width() / 4,
            y: renderer.height() / 3,
            vx: 3,
            vy: 2,
            radius: 40,
            color: Color::rgb(255, 70, 70),
        },
        Ball {
            x: renderer.width() / 2,
            y: renderer.height() / 2,
            vx: -2,
            vy: 3,
            radius: 30,
            color: Color::rgb(70, 255, 120),
        },
        Ball {
            x: renderer.width() * 3 / 4,
            y: renderer.height() * 2 / 3,
            vx: 2,
            vy: -2,
            radius: 50,
            color: Color::rgb(90, 140, 255),
        },
    ];

    let background = Color::rgb(7, 8, 16);
    let spec = Color::rgb(255, 255, 255);

    loop {
        let last_tick = interupts::timer_ticks();

        renderer.clear(background);

        for ball in &balls {
            renderer.draw_filled_circle(ball.x, ball.y, ball.radius, ball.color);
            renderer.draw_filled_circle(ball.x + ball.radius / 3, ball.y - ball.radius / 3, 5, spec);
        }

        for ball in &mut balls {
            ball.step(renderer.width(), renderer.height());
        }

        renderer.present(framebuffer_bytes);

        while interupts::timer_ticks() == last_tick {
            core::hint::spin_loop();
        }
    }
}

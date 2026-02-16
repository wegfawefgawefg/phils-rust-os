use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use libm::{cosf, powf, sinf, sqrtf};

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

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    fn len(self) -> f32 {
        sqrtf(self.dot(self))
    }

    fn norm(self) -> Self {
        let l = self.len();
        if l <= 0.000_01 {
            Self::new(0.0, 0.0, 0.0)
        } else {
            self.mul(1.0 / l)
        }
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

    fn fill(&mut self, color: Color) {
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
            _ => {
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
        }
    }

    fn fill_block(&mut self, x0: i32, y0: i32, w: i32, h: i32, color: Color) {
        for y in y0..(y0 + h) {
            for x in x0..(x0 + w) {
                self.set_pixel(x, y, color);
            }
        }
    }

    fn present(&self, target: &mut [u8]) {
        target.copy_from_slice(self.buffer);
    }
}

#[derive(Clone, Copy)]
struct Sphere {
    center: Vec3,
    radius: f32,
    albedo: Vec3,
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn to_color(v: Vec3) -> Color {
    Color::rgb(
        (clamp01(v.x) * 255.0) as u8,
        (clamp01(v.y) * 255.0) as u8,
        (clamp01(v.z) * 255.0) as u8,
    )
}

fn scene(tick: u64) -> [Sphere; 3] {
    let t = tick as f32 * 0.045;
    [
        Sphere {
            center: Vec3::new(
                1.9 * sinf(t * 0.9 + 0.2),
                1.1 * sinf(t * 1.3 + 1.2),
                -3.4 + 1.0 * sinf(t * 0.7 + 2.1),
            ),
            radius: 0.52,
            albedo: Vec3::new(1.0, 0.35, 0.35),
        },
        Sphere {
            center: Vec3::new(
                2.0 * cosf(t * 0.8 + 2.4),
                1.0 * sinf(t * 1.1 + 0.5),
                -3.2 + 1.15 * cosf(t * 0.6 + 1.1),
            ),
            radius: 0.48,
            albedo: Vec3::new(0.35, 1.0, 0.55),
        },
        Sphere {
            center: Vec3::new(
                1.7 * sinf(t * 1.05 + 4.0),
                1.2 * cosf(t * 0.95 + 0.9),
                -3.6 + 0.9 * sinf(t * 0.8 + 4.2),
            ),
            radius: 0.44,
            albedo: Vec3::new(0.4, 0.6, 1.0),
        },
    ]
}

fn ray_sphere_intersection(ro: Vec3, rd: Vec3, s: Sphere) -> Option<f32> {
    let oc = ro.sub(s.center);
    let a = rd.dot(rd);
    let b = 2.0 * oc.dot(rd);
    let c = oc.dot(oc) - s.radius * s.radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let sqrt_disc = sqrtf(disc);
    let inv_2a = 1.0 / (2.0 * a);
    let t0 = (-b - sqrt_disc) * inv_2a;
    let t1 = (-b + sqrt_disc) * inv_2a;

    if t0 > 0.0 {
        Some(t0)
    } else if t1 > 0.0 {
        Some(t1)
    } else {
        None
    }
}

fn raycast_spheres(ro: Vec3, rd: Vec3, spheres: &[Sphere; 3]) -> Option<(f32, usize)> {
    let mut best_t = f32::INFINITY;
    let mut best_id = 0usize;
    let mut hit = false;

    for (i, s) in spheres.iter().enumerate() {
        if let Some(t) = ray_sphere_intersection(ro, rd, *s) {
            if t < best_t {
                best_t = t;
                best_id = i;
                hit = true;
            }
        }
    }

    if hit {
        Some((best_t, best_id))
    } else {
        None
    }
}

fn shade_hit(ro: Vec3, rd: Vec3, t: f32, s: Sphere) -> Vec3 {
    let p = ro.add(rd.mul(t));
    let n = p.sub(s.center).norm();
    let light_pos = Vec3::new(0.0, 0.0, 2.5);
    let l = light_pos.sub(p).norm();
    let v = ro.sub(p).norm();
    let h = l.add(v).norm();

    let diff = clamp01(n.dot(l));
    let spec = powf(clamp01(n.dot(h)), 28.0) * 0.4;
    let amb = 0.08;
    let shade = amb + diff * 0.95;

    s.albedo.mul(shade).add(Vec3::new(spec, spec, spec))
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

    let render_w = (renderer.width() / 10).max(80);
    let render_h = (renderer.height() / 10).max(45);
    let block_w = (renderer.width() / render_w).max(1);
    let block_h = (renderer.height() / render_h).max(1);
    let aspect = render_w as f32 / render_h as f32;
    let camera = Vec3::new(0.0, 0.0, 2.8);
    let fov = 1.1f32;

    loop {
        let frame_tick = interupts::timer_ticks();
        let spheres = scene(frame_tick);
        renderer.fill(Color::rgb(0, 0, 0));
        let jitter = if (frame_tick & 1) == 0 { 0.25 } else { -0.25 };

        for py in 0..render_h {
            for px in 0..render_w {
                let uvx = ((px as f32 + 0.5 + jitter) / render_w as f32) * 2.0 - 1.0;
                let uvy = ((py as f32 + 0.5 - jitter) / render_h as f32) * 2.0 - 1.0;
                let rd = Vec3::new(uvx * aspect * fov, -uvy * fov, -1.0).norm();
                let c = if let Some((t_hit, id)) = raycast_spheres(camera, rd, &spheres) {
                    shade_hit(camera, rd, t_hit, spheres[id])
                } else {
                    let sky = 0.15 + 0.85 * clamp01(0.5 + 0.5 * rd.y);
                    Vec3::new(0.02, 0.03, 0.08).mul(sky)
                };
                renderer.fill_block(px * block_w, py * block_h, block_w, block_h, to_color(c));
            }
        }

        renderer.present(framebuffer_bytes);

        while interupts::timer_ticks() == frame_tick {
            core::hint::spin_loop();
        }
    }
}

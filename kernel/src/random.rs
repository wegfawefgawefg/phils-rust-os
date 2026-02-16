static mut RNG_STATE: u64 = 0xDEADBEEF;

pub fn pseudo_rand() -> u64 {
    unsafe {
        RNG_STATE = RNG_STATE.wrapping_mul(6364136223846793005).wrapping_add(1);
        RNG_STATE
    }
}

use core::ops::Range;

pub fn pseudo_rand_in_range_i32(range: Range<i32>) -> i32 {
    let len = range.end - range.start;
    if len == 0 {
        return range.start;
    }

    let rand_val = pseudo_rand() as i32;
    (rand_val % len) + range.start
}

pub fn pseudo_rand_in_range_u32(range: Range<u32>) -> u32 {
    let len = range.end - range.start;
    if len == 0 {
        return range.start;
    }

    let rand_val = (pseudo_rand() & 0xFFFF_FFFF) as u32;
    (rand_val % len) + range.start
}

pub fn pseudo_rand_in_range_i64(range: Range<i64>) -> i64 {
    let len = range.end - range.start;
    if len == 0 {
        return range.start;
    }

    let rand_val = pseudo_rand() as i64;
    (rand_val % len) + range.start
}

pub fn pseudo_rand_in_range_u64(range: Range<u64>) -> u64 {
    let len = range.end - range.start;
    if len == 0 {
        return range.start;
    }

    let rand_val = pseudo_rand();
    (rand_val % len) + range.start
}

pub fn pseudo_rand_in_range_f32(range: Range<f32>) -> f32 {
    let len = range.end - range.start;
    if len == 0.0 {
        return range.start;
    }

    let rand_val = (pseudo_rand() & 0xFFFF_FFFF) as f32 / u32::MAX as f32;
    (rand_val * len) + range.start
}

pub fn pseudo_rand_in_range_f64(range: Range<f64>) -> f64 {
    let len = range.end - range.start;
    if len == 0.0 {
        return range.start;
    }

    let rand_val = pseudo_rand() as f64 / u64::MAX as f64;
    (rand_val * len) + range.start
}

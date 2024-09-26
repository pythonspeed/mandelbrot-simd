//! Vectorized parallel Mandelbrot implementation
#![allow(non_camel_case_types)]

use crate::*;
use std::simd::prelude::*;

type u64s = u64x8;
type u32s = u32x8;
type f64s = f64x8;

/// Storage for complex numbers in SIMD format.
/// The real and imaginary parts are kept in separate registers.
#[derive(Copy, Clone)]
struct Complex {
    real: f64s,
    imag: f64s,
}

/// Returns the number of iterations it takes for the Mandelbrot sequence
/// to diverge at this point, or `ITER_LIMIT` if it doesn't diverge.
fn get_count(start: &Complex) -> u32s {
    let mut current = start.clone();
    let mut count = u64s::splat(0);
    let threshold_mask = f64s::splat(THRESHOLD);

    for _ in 0..ITER_LIMIT {
        let rr = current.real * current.real;
        let ii = current.imag * current.imag;

        // Keep track of those lanes which haven't diverged yet. The other ones
        // will be masked off.
        let undiverged_mask = (rr + ii).simd_le(threshold_mask);

        // Stop the iteration if they all diverged.
        if !undiverged_mask.any() {
            break;
        }

        // For undiverged lanes add 1, for diverged lanes add 0.
        count += undiverged_mask.select(u64s::splat(1), u64s::splat(0));

        let ri = current.real * current.imag;

        current.real = start.real + (rr - ii);
        current.imag = start.imag + (ri + ri);
    }
    count.cast()
}

pub fn generate(dims: Dimensions, xr: Range, yr: Range) -> Vec<u32> {
    let (width, height) = dims;

    let block_size = f64s::LEN;

    assert_eq!(
        width % block_size,
        0,
        "image width = {} is not divisible by the number of vector lanes = {}",
        width,
        block_size,
    );

    let width_in_blocks = width / block_size;

    // The initial X values are the same for every row.
    let xs = unsafe {
        let dx = (xr.end - xr.start) / (width as f64);
        let mut buf: Vec<f64s> = vec![f64s::splat(0.); width_in_blocks];

        std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut f64, width)
            .iter_mut()
            .enumerate()
            .for_each(|(j, x)| {
                *x = xr.start + dx * (j as f64);
            });

        buf
    };

    let dy = (yr.end - yr.start) / (height as f64);

    let len = width_in_blocks * height;
    let mut out = Vec::with_capacity(len);
    unsafe {
        out.set_len(len);
    }

    out.par_chunks_mut(width_in_blocks)
        .enumerate()
        .for_each(|(i, row)| {
            let y = f64s::splat(yr.start + dy * (i as f64));
            row.iter_mut().enumerate().for_each(|(j, count)| {
                let x = xs[j];
                let z = Complex { real: x, imag: y };
                *count = get_count(&z);
            });
        });

    // This is safe, we're transmuting from a more-aligned type to a
    // less-aligned one.
    #[allow(clippy::unsound_collection_transmute)]
    unsafe {
        let mut out: Vec<u32> = std::mem::transmute(out);
        out.set_len(width * height);
        out
    }
}

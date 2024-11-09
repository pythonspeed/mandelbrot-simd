//! Vectorized parallel Mandelbrot implementation, using the `wide` crate.
#![allow(non_camel_case_types)]

use crate::*;
use wide::*;

// Instead of storing a single value, we store a vector of 8 values; each index
// in the vector, from 0 to 7, is a different "lane". Because they're special
// SIMD types, they'll get processed as a batch if the CPU supports it, using
// special SIMD "registers" (CPU storage slots), which can be 128-bit, 256-bit,
// or 512-bit.
type u64s = u64x4;
type u32s = u32x4;
type f64s = f64x4;

/// Storage for complex numbers in SIMD format. The real and imaginary parts are
/// kept in separate registers.
#[derive(Clone)]
struct Complex {
    real: f64s,
    imag: f64s,
}

/// Returns the number of iterations it takes for the Mandelbrot sequence
/// to diverge at this point, or `ITER_LIMIT` if it doesn't diverge.
fn get_count(start: &Complex) -> u32s {
    // Instead of getting a single value as input, we get a batch of 4.
    let mut current = start.clone();
    // Instead of having a single return value, we are going to return a batch
    // of 4 values. We initialize all of them to 0.
    let mut count = f64s::splat(0.0);
    // This is the threshold we will compare our current 4 values to, turned
    // into an SIMD vector of 4 values.
    let threshold_mask = f64s::splat(THRESHOLD);

    for _ in 0..ITER_LIMIT {
        // This looks just like the scalar operations, but we're multiplying 4
        // values against 4 values. Hopefully our CPU has the instructions that
        // less do all of these multiplications at once!
        let rr = current.real * current.real;
        let ii = current.imag * current.imag;

        // Keep track of those lanes which haven't diverged yet. The other ones
        // will be masked off.
        //
        // For example:
        //
        // [2.3, 4.2, 4.0, 4.1, 0.5, 0.7, 2.3, 5.0]
        //                 simd_le
        // [4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0]
        //                    ↓
        // [  1,   0,   1,   0,   1,   1,   1,   0]
        let undiverged_mask = (rr + ii).cmp_le(threshold_mask);

        // Stop the iteration if they all diverged.
        if !undiverged_mask.any() {
            break;
        }

        // For undiverged lanes add 1, for diverged lanes add 0.
        count += undiverged_mask.blend(f64s::splat(1.0), f64s::splat(0.0));

        // Same calculation as scalar algorithm, but hopefully doing 4
        // operations in a single SIMD CPU instruction.
        let ri = current.real * current.imag;

        current.real = start.real + (rr - ii);
        current.imag = start.imag + (ri + ri);
    }
    let mut result = [0u32; 4];
    for (i, value) in count.to_array().into_iter().enumerate() {
        result[i] = *value as u32;
    }
    u32s::from(result)
}

pub fn generate(dims: Dimensions, xr: Range, yr: Range) -> Vec<u32> {
    let (width, height) = dims;

    let block_size = 4;

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

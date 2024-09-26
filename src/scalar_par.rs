//! Scalar mandelbrot implementation

use crate::*;

/// Complex number
#[repr(align(16))]
#[derive(Copy, Clone)]
struct Complex {
    real: f64,
    imag: f64,
}

/// Returns the number of iterations it takes for the Mandelbrot sequence
/// to diverge at this point, or `ITER_LIMIT` if it doesn't diverge.
fn get_count(start: Complex) -> u32 {
    let mut current = start.clone();
    for iteration in 0..ITER_LIMIT {
        let rr = current.real.powi(2);
        let ii = current.imag.powi(2);
        if rr + ii > THRESHOLD {
            return iteration;
        }
        let ri = current.real * current.imag;

        current.real = start.real + (rr - ii);
        current.imag = start.imag + (ri + ri);
    }
    ITER_LIMIT
}

pub fn generate(dims: Dimensions, xr: Range, yr: Range) -> Vec<u32> {
    let (width, height) = dims;

    let xs = {
        let dx = (xr.end - xr.start) / (width as f64);

        let mut buf = Vec::new();

        (0..width)
            .into_par_iter()
            .map(|j| xr.start + dx * (j as f64))
            .collect_into_vec(&mut buf);

        buf
    };

    let dy = (yr.end - yr.start) / (height as f64);

    let len = width * height;
    let mut out = Vec::with_capacity(len);
    unsafe {
        out.set_len(len);
    }

    out.par_chunks_mut(width).enumerate().for_each(|(i, row)| {
        let y = yr.start + dy * (i as f64);
        row.iter_mut().enumerate().for_each(|(j, count)| {
            let x = xs[j];
            let z = Complex { real: x, imag: y };
            *count = get_count(z) as u32;
        });
    });

    out
}

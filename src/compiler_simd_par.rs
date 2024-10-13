//! Scalar mandelbrot implementation

use crate::*;

/// Complex number
#[repr(align(16))]
#[derive(Copy, Clone)]
struct Complex {
    real: f64,
    imag: f64,
}

const BATCH_SIZE:usize=8;

/// Returns the number of iterations it takes for the Mandelbrot sequence
/// to diverge at this point, or `ITER_LIMIT` if it doesn't diverge.
fn set_count(start: [Complex;BATCH_SIZE], counts:&mut[u32]) {
    let mut current = start.clone();
    let mut rr=[0_f64;BATCH_SIZE];
    let mut ii=[0_f64;BATCH_SIZE];
    let mut ri=[0_f64;BATCH_SIZE];
    for i in 0..BATCH_SIZE {
        counts[i]=ITER_LIMIT;
    }
    for iteration in 0..ITER_LIMIT {
        for i in 0..BATCH_SIZE {
            if counts[i]==ITER_LIMIT {
            //if rr[i] + ii[i] <= THRESHOLD {
                rr[i] = current[i].real * current[i].real;
                ii[i] = current[i].imag * current[i].imag;
                if rr[i] + ii[i] > THRESHOLD {
                    counts[i] = iteration;
                }
                ri[i] = current[i].real * current[i].imag;

                current[i].real = start[i].real + (rr[i] - ii[i]);
                current[i].imag = start[i].imag + (ri[i] + ri[i]);
            }
        }
    }

}

pub fn generate(dims: Dimensions, xr: Range, yr: Range) -> Vec<u32> {
    let (width, height) = dims;

    let xs = {
        let dx = (xr.end - xr.start) / (width as f64);

        let mut buf:Vec<f64> = Vec::new();

        buf=(0..width)
            .into_iter()
            .map(|j| xr.start + dx * (j as f64))
            .collect();

        buf
    };

    let dy = (yr.end - yr.start) / (height as f64);

    let len = width * height;
    let mut out = Vec::with_capacity(len);
    //let mut out = vec![ITER_LIMIT;len];
    unsafe {
        out.set_len(len);
    }


    out.chunks_mut(width).enumerate().for_each(|(i, row)| {
        let y = yr.start + dy * (i as f64);
        let mut zs=[Complex{real:0_f64,imag:0_f64};BATCH_SIZE];
        row.chunks_exact_mut(BATCH_SIZE).enumerate().for_each(|(j, count)| {
            for i in 0..BATCH_SIZE {
                zs[i].real=xs[j*BATCH_SIZE+i];
                zs[i].imag=y;
            }
            //let x = xs[j];
            //let z = Complex { real: x, imag: y };
            set_count(zs,count);
        });
    });

    out
}

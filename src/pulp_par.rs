//! Vectorized parallel Mandelbrot implementation
#![allow(non_camel_case_types)]

use std::mem::MaybeUninit;

use crate::*;
use pulp::Simd;

/// Returns the number of iterations it takes for the Mandelbrot sequence
/// to diverge at this point, or `ITER_LIMIT` if it doesn't diverge.
#[inline(always)]
fn get_count<S: Simd>(simd: S, real: S::f64s, imag: S::f64s) -> S::u64s {
    let mut cur_re = real;
    let mut cur_im = imag;

    let mut count = simd.splat_u64s(0);
    let threshold_mask = simd.splat_f64s(THRESHOLD);

    for _ in 0..ITER_LIMIT {
        let rr = simd.mul_f64s(cur_re, cur_re);
        let ii = simd.mul_f64s(cur_im, cur_im);

        let undiverged_mask = simd.less_than_or_equal_f64s(simd.add_f64s(rr, ii), threshold_mask);

        if simd.first_true_m64s(undiverged_mask) == S::F64_LANES {
            break;
        }
        count = simd.select_u64s_m64s(
            undiverged_mask,
            simd.add_u64s(count, simd.splat_u64s(1)),
            count,
        );

        let ri = simd.mul_f64s(cur_re, cur_im);
        cur_re = simd.add_f64s(real, simd.sub_f64s(rr, ii));
        cur_im = simd.add_f64s(imag, simd.add_f64s(ri, ri));
    }
    count
}

pub fn generate(dims: Dimensions, xr: Range, yr: Range) -> Vec<u32> {
    struct Impl {
        dims: Dimensions,
        xr: Range,
        yr: Range,
    }
    impl pulp::WithSimd for Impl {
        type Output = Vec<u32>;

        fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
            let Self { dims, xr, yr } = self;
            let block_size = S::F64_LANES;
            let (width, height) = dims;

            assert_eq!(
                width % block_size,
                0,
                "image width = {} is not divisible by the number of vector lanes = {}",
                width,
                block_size,
            );

            let width_in_blocks = width / block_size;

            // The initial X values are the same for every row.
            let xs = {
                let dx = (xr.end - xr.start) / (width as f64);
                let mut buf = vec![simd.splat_f64s(0.0); width_in_blocks];

                bytemuck::cast_slice_mut(&mut buf)
                    .iter_mut()
                    .enumerate()
                    .for_each(|(j, x)| {
                        *x = xr.start + dx * (j as f64);
                    });

                buf
            };

            let dy = (yr.end - yr.start) / (height as f64);

            // let mut out = vec![0u32; width * height];
            let mut out = Vec::with_capacity(width * height);

            out.spare_capacity_mut()[..width * height]
                .par_chunks_mut(width)
                .enumerate()
                .for_each(|(i, row)| {
                    let y = simd.splat_f64s(yr.start + dy * (i as f64));

                    struct Impl<'a, S: Simd> {
                        simd: S,
                        x: &'a [S::f64s],
                        y: S::f64s,
                        count: &'a mut [MaybeUninit<u32>],
                    }

                    impl<S: Simd> pulp::NullaryFnOnce for Impl<'_, S> {
                        type Output = ();

                        #[inline(always)]
                        fn call(self) -> Self::Output {
                            let Self { simd, x, y, count } = self;
                            for (&x, count) in
                                core::iter::zip(x, count.chunks_exact_mut(S::F64_LANES))
                            {
                                let tmp = get_count(simd, x, y);
                                let tmp =
                                    bytemuck::cast_slice::<_, u64>(core::slice::from_ref(&tmp));
                                for (dst, src) in core::iter::zip(count, tmp) {
                                    *dst = MaybeUninit::new(*src as u32);
                                }
                            }
                        }
                    }

                    simd.vectorize(Impl {
                        simd,
                        x: &xs,
                        y,
                        count: row,
                    });
                });

            unsafe { out.set_len(width * height) };

            out
        }
    }

    let arch = pulp::Arch::new();
    arch.dispatch(Impl { dims, xr, yr })
}

# Mandelbrot

This is based on the [`mandelbrot` benchmark from the benchmarksgame][bg], as implemented in the defunct `packed_simd` project, and soon-to-be-updated for Rust portable SIMD (nightly only at the moment).

Licensed under MIT or Apache 2.0, at your choice, copyrighted by the Rust Project Developers and (for the port to portable SIMD parts) Itamar Turner-Trauring.

## Background

http://mathworld.wolfram.com/MandelbrotSet.html

## Usage

It takes four arguments in this order:

* `width`: width of the image to render
* `height`: height of the image to render
* `algorithm`: algorithm to use:
  * `scalar`: scalar algorithm
  * `simd`: parallelized SIMD algorithm
  * `ispc`: ISPC + tasks algorithm
* `--color` (optional): enables colorized output, which also determines the image format.
  * disabled (default): PBM: Portable BitMap format (black & white output)
  * enabled: PPM: Portable PixMap format (colored output)

The resulting image is piped to `stdout`.

`cargo run --release -- 400 400 --algo simd > output.ppm` outputs:

![run_400_png](https://user-images.githubusercontent.com/904614/43190942-72bdb834-8ffa-11e8-9dcf-a9a9632ae907.png)

`cargo run --release -- 400 400 --algo simd --color > output.ppm` outputs:

![run_400_400_1_1_png](https://user-images.githubusercontent.com/904614/43190948-759969a4-8ffa-11e8-81a9-35e5baef3e86.png)

## Performance

Make sure you have `hyperfine` installed.

Multi-threaded:
```
$ ./benchmark.sh
```

Single-threaded:

```
$ RAYON_NUM_THREADS=1 ./benchmark.sh
```

[bg]: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/mandelbrot.html#mandelbrot

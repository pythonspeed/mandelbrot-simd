# Mandelbrot

This repository includes multiple implementations of the Mandelbrot algorithm in Rust, showing how parallelism and SIMD can be used to speed it up.

From fastest to slowest (keeping in mind this doesn't mean this will be the ordering for _your_ problem, and the order may be different on different hardware):

* **`std::simd`**: Uses the built-in portable SIMD support in Rust, requires unstable compiler.
* **`pulp` SIMD:** Use the [`pulp`](https://docs.rs/pulp/) crate to implement portable SIMD operations. **`pulp` can use the stable compiler.**
* **`wide` SIMD**: Use the [`wide`](https://docs.rs/wide/) crate to implement portable SIMD operations. **`wide` can use the stable compiler.**
* **Scalar with autovectorization:** A scalar version written so the compiler can autovectorize it, i.e. decide to use SIMD automatically.
* **Scalar:** The standard algorithm, one value at a time.

All implementations use parallelism out of the box, using Rayon.

## History

Much of this is based on the [`mandelbrot` benchmark from the benchmarksgame][bg], as implemented in the since-superseded [`packed_simd` project](https://github.com/rust-lang/packed_simd/tree/master/examples/mandelbrot).

Changes from the original implementation:

* Updated to use the newer [Rust portable SIMD API](https://doc.rust-lang.org/std/simd/index.html) (nightly only at the moment).
* Simplified the calculation logic so it's easier to understand.
* Autovectorized implementation contributed by [giovannicuccu](https://github.com/giovannicuccu).
* `pulp` implementation contributed by [sarah quiñones](https://github.com/sarah-quinones).

Licensed under MIT or Apache 2.0, at your choice, copyrighted by the Rust Project Developers, with minor changes by Itamar Turner-Trauring.

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

# w4-pnger

A command line tool to aid in working with png files with [WASM-4](https://github.com/aduros/wasm4).

Using the CLI, you're able to process png files into compressed WASM-4 image data. The png-to-WASM-4 routine produces identical results to the `png2src` tool in WASM-4.

To use the CLI, run `w4-pnger convert [PNG File Pattern] {--rs | --raw | --text {OUTPUT FILE PREFIX}} {--compress | -c}` to generate the output. W4-pnger currently supports outputting to text, and raw files. 

To open these raw files from within a WASM-4 application, add
```
w4-tiny-decomp = { git = "https://github.com/fishtaco567/w4-pnger" }
```
to your cargo.toml, then create a `Decompressor` object with `Decompressor::new(buf)`, where buf is an `&mut [u8]` large enough to hold your largest decompressed image. Then call `decompress`, which will return a `Result<SpriteHandle, &str>`. The decompressor may not be used again until this `SpriteHandle` is dropped.
use png::Reader;
use std::fs::File;

use crate::compress::pkcomp::PkComp;
use crate::compress::Compressor;
use crate::pngstream::PngStream;
use crate::wasm4png::W4Sprite;

pub struct Analyzer<'a> {
    path: &'a str,
}

impl<'a> Analyzer<'a> {
    pub fn new(path: &'a str) -> Self {
        Analyzer { path }
    }

    pub fn run(self) {
        println!("Analyzing images...");
        let stream = PngStream::new(self.path);

        for png_res in stream {
            match png_res {
                Ok((name, png)) => process_png(name, png),
                Err(e) => eprintln!("{}, continuing with other files", e),
            }
        }
    }
}

fn process_png(image_name: String, png_reader: Reader<File>) {
    println!("Analyzing {image_name}...");

    let mut png_reader = png_reader;
    match W4Sprite::from_reader(&mut png_reader) {
        Ok(png) => {
            let png_bytes = png.get_bytes();

            let compressor = PkComp {};

            match compressor.compress(&png_bytes) {
                Ok(compressed) => {
                    let png_size = png_bytes.len();
                    let compressed_size = compressed.total_size;
                    let compression_method = compressed.readable_compression_name;
                    let compression_statistics = compressed.readable_compression_statistics;

                    println!("\nSprite {image_name} is {png_size}B in WASM-4 native format, and can be compressed to {compressed_size}B.\n\
                    Compression method: {compression_method}\n\
                    Statistics: {compression_statistics}");
                }
                Err(e) => eprintln!("Error encountered compressing sprite {image_name}: {e}"),
            }
        }
        Err(e) => eprintln!("Encountered error processing sprite {image_name}: {e}"),
    }
}

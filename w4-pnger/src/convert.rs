use png::Reader;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::compress::pkcomp::PkComp;
use crate::compress::{CompType, Compressor};
use crate::pngstream::PngStream;
use crate::wasm4png::W4Sprite;

pub struct Converter<'a> {
    path: &'a str,
    name: &'a str,
    out_type: OutputType,
    compress: bool,
}

impl<'a> Converter<'a> {
    pub fn new(path: &'a str, name: &'a str, out_type: OutputType, compress: bool) -> Self {
        Converter {
            path,
            name,
            out_type,
            compress,
        }
    }

    pub fn run(self) {
        let stream = PngStream::new(self.path);

        for png_res in stream {
            match png_res {
                Ok((name, png)) => self.process_png(name, png),
                Err(e) => eprintln!("{}, continuing with other files", e),
            }
        }
    }

    fn process_png(&self, image_name: String, png_reader: Reader<File>) {
        let mut png_reader = png_reader;
        match W4Sprite::from_reader(&mut png_reader) {
            Ok(png) => {
                let png_bytes = png.get_bytes();

                let out_bytes = if self.compress {
                    let mut compressor = PkComp {};
                    let mut compressed = compressor.compress(&png_bytes).unwrap();

                    println!(
                        "Compressed {} with {}, from {} bytes to {} bytes, ({:04.2} %)",
                        image_name,
                        &compressed.readable_compression_name,
                        png_bytes.len(),
                        compressed.total_size,
                        (png_bytes.len() as f32 / compressed.total_size as f32) * 100.0
                    );

                    let mut out = vec![CompType::Pk as u8];
                    out.append(&mut compressed.header_bytes);
                    out.append(&mut compressed.content_bytes);
                    out
                } else {
                    let mut out = vec![CompType::Uncompressed as u8];
                    out.append(&mut png_bytes.clone());
                    out
                };

                match self.out_type {
                    OutputType::Rust => {}
                    OutputType::Raw => {
                        let out_name = self.name.to_owned() + "_" + &image_name + ".ws";
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(&out_name);

                        match file {
                            Ok(mut f) => {
                                _ = f.write_all(&out_bytes);
                            }
                            Err(e) => eprint!("{}", e),
                        }
                    }
                    OutputType::Text => {
                        let out_name = self.name.to_owned() + "_" + &image_name + ".txt";
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(&out_name);

                        match file {
                            Ok(f) => {
                                let mut buf_write = BufWriter::new(f);

                                _ = buf_write.write(
                                    format!(
                                        "width: {}\nheight: {}\nbpp: {}\ndata: ",
                                        png.width, png.height, png.bpp as u8
                                    )
                                    .as_bytes(),
                                );
                                _ = buf_write.write(format!("{:02X?}", &out_bytes).as_bytes());
                            }
                            Err(e) => eprint!("{}", e),
                        }
                    }
                }
            }
            Err(e) => eprintln!("{}, continuing with other errors", e),
        }
    }
}

pub enum OutputType {
    Rust,
    Raw,
    Text,
}

impl OutputType {
    pub fn from_str(from: &str) -> OutputType {
        match from {
            "rs" => OutputType::Rust,
            "raw" => OutputType::Raw,
            "text" => OutputType::Text,
            _ => panic!("Invalid output type"),
        }
    }
}

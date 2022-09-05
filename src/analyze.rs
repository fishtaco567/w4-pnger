use png::Reader;
use std::fs::File;

use crate::pngstream::PngStream;

struct OutputData {
    total_size: usize,
}

pub struct Analyzer<'a> {
    path: &'a str,
}

impl<'a> Analyzer<'a> {
    pub fn new(path: &'a str) -> Self {
        Analyzer { path }
    }

    pub fn run(self) {
        let stream = PngStream::new(self.path);

        for png_res in stream {
            match png_res {
                Ok((name, png)) => process_png(name, png),
                Err(e) => eprintln!("{}, continuing with other files", e),
            }
        }
    }
}

fn process_png(image_name: String, png: Reader<File>) {}

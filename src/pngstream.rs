use std::{
    fs::File,
    io::BufReader,
    path::{Iter, PathBuf},
};

use glob::{glob, Paths};
use png::{Decoder, Reader};

use anyhow::{anyhow, Result};

pub struct PngStream {
    paths: Paths,
}

impl PngStream {
    pub fn new(path: &str) -> Self {
        PngStream {
            paths: glob(path).expect("Must be a valid pattern"),
        }
    }
}

impl Iterator for PngStream {
    type Item = Result<Reader<File>>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = match self.paths.next()? {
            Ok(p) => p,
            Err(e) => return Some(Err(anyhow!(e))),
        };

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Some(Err(anyhow!(e))),
        };

        match Decoder::new(file).read_info() {
            Ok(r) => Some(Ok(r)),
            Err(e) => return Some(Err(anyhow!(e))),
        }
    }
}
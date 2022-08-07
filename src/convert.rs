use std::path::PathBuf;

pub struct Converter<'a> {
    path: &'a PathBuf,
    name: &'a str,
    out_type: OutputType,
    compress: bool,
}

impl<'a> Converter<'a> {
    pub fn new(path: &'a PathBuf, name: &'a str, out_type: OutputType, compress: bool) -> Self {
        Converter {
            path: path,
            name: name,
            out_type: out_type,
            compress: compress,
        }
    }

    pub fn run(self) {

    }
}

pub enum OutputType {
    Rust,
    Raw,
}

impl OutputType {
    pub fn from_str(from: &str) -> OutputType {
        match from {
            "rs" => OutputType::Rust,
            "raw" => OutputType::Raw,
            _ => panic!("Invalid output type"),
        }
    }
}

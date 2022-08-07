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

    pub fn run(self) {}
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

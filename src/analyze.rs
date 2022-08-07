use std::path::PathBuf;

pub struct Analyzer<'a> {
    path: &'a PathBuf,
}

impl<'a> Analyzer<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        Analyzer { path: path }
    }

    pub fn run(self) {}
}

pub struct Analyzer<'a> {
    path: &'a str,
}

impl<'a> Analyzer<'a> {
    pub fn new(path: &'a str) -> Self {
        Analyzer { path }
    }

    pub fn run(self) {}
}

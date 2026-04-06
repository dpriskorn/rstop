pub struct SortMode {
    pub sort_by_mem: bool,
}

impl SortMode {
    pub fn new() -> Self {
        SortMode { sort_by_mem: false }
    }

    pub fn toggle(&mut self) {
        self.sort_by_mem = !self.sort_by_mem;
    }
}

impl Default for SortMode {
    fn default() -> Self {
        Self::new()
    }
}

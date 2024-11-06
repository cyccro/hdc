#[derive(Debug)]
pub struct Cursor {
    line: usize,
    column: usize,
    idx: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 0,
            idx: 0,
        }
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
    pub fn index(&self) -> usize {
        self.idx
    }
    pub fn advance(&mut self) -> usize {
        self.column += 1;
        self.idx += 1;
        self.idx
    }
    pub fn backward(&mut self) -> usize {
        self.column -= 1;
        self.idx -= 1;
        self.idx
    }
    pub fn advance_line(&mut self) -> usize {
        self.line += 1;
        self.column = 0;
        self.idx += 1;
        self.idx
    }
}

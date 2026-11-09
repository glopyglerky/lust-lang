pub struct Table {
    cells: Vec<u32>,
}

impl Table {
    pub fn new(n: u32) -> Table {
        println!("{}:{}", file!(), line!());
        let mut cells = Vec::new();
        for i in 0..n {
            cells.push(i * 10);
        }
        Table { cells }
    }

    pub fn get(&self, i: usize) -> u32 {
        self.cells[i]
    }
}

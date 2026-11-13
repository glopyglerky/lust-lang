// Infer edits in a fn that returns impl Iterator (ts_goport core.rs nodes): v4 is a body error
// (u64: Sum<&u8>), so the hidden type is gone in the cold program; the warm daemon keeps the
// program it had before v4 and the revert v5 reuses it (kept=hidden_type took a full collect).
mod m { pub const R: u32 = 1; }
struct T { slots: Vec<u32>, entries: Vec<u8> }
impl T {
    pub fn nodes(&self) -> impl Iterator<Item = (usize, usize, &u8)> {
        // slot
        let _lust_i = [7; 2]; let _lust_s: u32 = _lust_i.iter().sum();
        self.slots
            .iter()
            .enumerate()
            .filter(|&(_, &offset)| offset != 9)
            .map(|(index, &offset)| {
                let entry = offset as usize;
                (index, entry, &self.entries[entry])
            })
    }
}
fn main() {
    let t = T { slots: vec![0, 9, 1], entries: vec![5, 6] };
    for (a, b, c) in t.nodes() { println!("{a} {b} {c}"); }
    println!("{}", m::R);
}

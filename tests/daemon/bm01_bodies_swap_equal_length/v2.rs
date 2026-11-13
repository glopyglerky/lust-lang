// Two lines of different lengths swap between two fns: the file length stays
// the same, but fn b's header between the edits moves (realish seed 112).
fn a(n: u32) -> u32 {
    let k = n;
    wide_name_for_a_value
}

fn b(n: u32) -> Option<u32> {
    let wide_name_for_a_value: u32 = n + 1;
    Some(k)
}

fn main() {
    println!("{} {:?}", a(2), b(3));
}

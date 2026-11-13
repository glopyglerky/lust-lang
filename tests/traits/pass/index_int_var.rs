// ARCH 2.4 step 3 and 2.12: `s[x]` selects Index<usize>, so the
// integer variable x is fixed to usize at the index point.
// From the Opus probe idx.rs.
fn main() {
    let x = 0;
    let s = vec![1u8, 2, 3];
    let a = s[x];
    let y = x.wrapping_sub(1);
    println!("{} {}", a, y);
    // Same shape on a plain array.
    let i = 1;
    let arr = [10u8, 20, 30];
    println!("{} {}", arr[i], i.wrapping_add(1));
}

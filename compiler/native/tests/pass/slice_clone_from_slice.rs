// [T]::clone_from_slice (ts-rust core.rs:1779): element-wise clone_from for a
// non-Copy T; a TrivialClone T takes copy_from_slice (its panic message too).
#[derive(Clone, Copy, Debug)]
struct D(u8);
fn main() {
    let mut a = vec![String::from("a"); 3];
    let b = vec![String::from("x"), String::from("y")];
    a[1..3].clone_from_slice(&b);
    let mut c = [D(1), D(2)];
    c.clone_from_slice(&[D(7), D(8)]);
    let mut v = vec![vec![1u8; 4]; 2];
    v.clone_from_slice(&[vec![9u8], vec![]]);
    println!("{:?} {:?} {:?}", a, c, v);
    let mut d = vec![String::new(); 2];
    d.clone_from_slice(&b[..1]);
}

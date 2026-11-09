// expect: L0104
// rustc E0277: no From<[u8; 2]> for Vec<i32>.
fn main() {
    let x: Vec<i32> = [1u8, 2].into();
    println!("{x:?}");
}

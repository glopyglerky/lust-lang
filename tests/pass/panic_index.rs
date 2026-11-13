// Out-of-bounds index panics.
fn main() {
    let v = vec![1, 2, 3];
    let mut i = 0;
    while i < 5 {
        println!("{}", v[i]);
        i += 1;
    }
}

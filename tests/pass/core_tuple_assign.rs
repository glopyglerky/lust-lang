fn main() {
    let (mut a, mut b) = (1, 2);
    (a, b) = (b, a);
    let mut arr = [0; 2];
    [arr[0], arr[1]] = [5, 6];
    let mut t = (0, 0);
    (t.0, _) = (9, 10);
    println!("{} {} {:?} {:?}", a, b, arr, t);
}

// `sum` and `product` overflow inside core's `integer_sum_product!`.
// panic: attempt to multiply with overflow
fn main() {
    let n = std::env::args().count();
    let v = vec![100i32; 4 + n];
    println!("{}", v.iter().sum::<i32>());
    let w: Vec<u8> = vec![2, 3, 4];
    println!("{}", w.iter().product::<u8>());
    println!("{}", v.into_iter().product::<i32>());
}

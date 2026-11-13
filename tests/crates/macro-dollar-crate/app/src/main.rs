use mid::count;

fn main() {
    let s = mid::sum_ones!(7, 8, 9);
    let o = mid::outer!();
    let n = count!(1, 2, 3, 4);
    println!("{} {} {}", s.0, o, n);
}

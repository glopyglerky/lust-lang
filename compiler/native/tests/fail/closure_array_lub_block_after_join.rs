// expect: L0110
// rustc E0308: the hint for later elements stays the first element's type,
// even after two closures joined as a fn pointer.
fn main() {
    let c = || 1;
    let d = || 2;
    let arr = [c, d, if true { || 3 } else { || 4 }];
    println!("{}", arr[2]());
}

// `unsafe_code` at forbid needs no lint pass: lust rejects `unsafe`
// itself.
#![forbid(unsafe_code)]

fn main() {
    println!("hi");
}

mod a;
mod b;

fn helper() -> i64 {
    100
}

fn main() {
    let v = a::value() + a::inner::value() + b::value() + helper();
    println!("mods {}", v);
}

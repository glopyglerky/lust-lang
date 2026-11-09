// A `-> &'static str` closure result outlives the scope it was called in.
fn main() {
    let get = || -> &'static str { "lit" };
    let r;
    { let _t = String::from("t"); r = get(); }
    println!("{}", r);
}

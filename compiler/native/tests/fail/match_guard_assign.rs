// expect: L0309
struct D(u8);

fn main() {
    let o = Some(D(5));
    match o {
        Some(mut x) if {
            x.0 = 9;
            true
        } =>
        {
            println!("got {}", x.0)
        }
        _ => println!("other"),
    }
}

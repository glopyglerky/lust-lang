// expect: L0308
struct D(u8);

impl D {
    fn bump(&mut self) {
        self.0 += 1;
    }
}

fn main() {
    let o = Some(D(5));
    match o {
        Some(mut x) if {
            x.bump();
            true
        } =>
        {
            println!("got {}", x.0)
        }
        _ => println!("other"),
    }
}

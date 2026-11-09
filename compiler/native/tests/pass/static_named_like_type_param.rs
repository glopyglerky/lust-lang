// A `static T` (value namespace) does not hide a type parameter `T`,
// in the same module or another one (rustc keeps the namespaces apart;
// W-STD-C: std source modules use `T` and `F` params).
mod m {
    pub struct W<T> {
        pub v: Option<T>,
    }
}

struct Pair<F> {
    f: Option<F>,
}

static T: u8 = 1;
static F: u16 = 2;

fn main() {
    let w = m::W::<u8> { v: None };
    let p: Pair<u32> = Pair { f: Some(3) };
    println!("{} {} {} {:?}", w.v.is_none(), T, F, p.f);
}

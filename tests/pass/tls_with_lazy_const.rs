// thread_local! in both initializer forms: a lazy value is made at its
// first access on the thread (and only then), a const one never runs code.
// Several statics in one call, attributes, visibility, and the path form.
struct Named(&'static str);

std::thread_local! {
    /// A documented lazy key.
    #[allow(dead_code)]
    pub static LAZY: Named = {
        println!("init lazy");
        Named("lazy")
    };
    static EAGER: u32 = const { 6 * 7 };
    pub(crate) static UNUSED: Vec<u8> = {
        println!("never runs");
        Vec::new()
    }
}

thread_local!(static PAREN: (u8, char) = (1, 'x'));

fn main() {
    println!("start");
    let n = EAGER.with(|v| *v + 1);
    println!("eager {}", n);
    LAZY.with(|v| println!("first {}", v.0));
    LAZY.with(|v| println!("second {}", v.0));
    let doubled = LAZY.with(|v| v.0.len() * 2);
    println!("doubled {}", doubled);
    match EAGER.try_with(|v| *v) {
        Ok(v) => println!("try_with ok {}", v),
        Err(_) => println!("try_with err"),
    }
    let owned = String::from("captured");
    PAREN.with(move |p| println!("{} {} {}", p.0, p.1, owned));
}

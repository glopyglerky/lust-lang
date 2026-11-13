// `let`-`else` temporaries: an extending borrow in the initializer lives
// to the end of the block; other temporaries drop at the end of the
// statement on success, and before the `else` block on failure.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn take(o: Option<&D>) -> u32 {
    let Some(x) = o else {
        println!("none");
        return 0;
    };
    x.0.len() as u32
}
fn none(_d: &D) -> Option<u8> {
    None
}
fn fail_path() {
    let Some(_x) = none(&mk("t")) else {
        println!("else");
        return;
    };
    println!("unreachable");
}
fn main() {
    {
        let Some(x) = Some(&mk("t")) else { return };
        println!("ctor {}", x.0);
    }
    {
        let Some(x) = &Some(mk("t")) else { return };
        println!("ref {}", x.0);
    }
    {
        let Some(_) = &Some(mk("t")) else { return };
        println!("ref wild");
    }
    {
        let (Some(_), y) = (Some(mk("t")), mk("y")) else { return };
        println!("tuple {}", y.0);
    }
    {
        let Some(x) = (mk("t"), Some(1u8)).1 else { return };
        println!("ok path {}", x);
    }
    fail_path();
    println!("{}", take(None) + take(Some(&D("d"))));
}

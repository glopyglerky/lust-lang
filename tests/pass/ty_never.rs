// The never type in written signatures (S5): parsing it is required
// for the first source std fn (`std::process::exit` returns `!`), and
// check already unified it. A never fn composes in value position.
fn fail() -> ! {
    panic!("no")
}

fn pick(yes: bool) -> i32 {
    if yes {
        4
    } else {
        fail()
    }
}

fn main() {
    println!("picked {}", pick(true));
}

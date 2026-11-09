// A user fn named `exit` and the source std fn `std::process::exit`
// coexist: the spliced body rides under a mangled name (S5), so it
// cannot collide with the user's plain name.
fn exit() {
    println!("user exit");
}

fn main() {
    exit();
    std::process::exit(0);
    println!("unreachable");
}

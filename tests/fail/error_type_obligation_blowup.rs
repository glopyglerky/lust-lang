// expect: L0101
// rustc E0425: `target` is not declared. Its error type matched every impl header in trait
// selection (`&{error}: AsRef<Path>` picked the blanket `&T` impl with `T` unbound), so these
// calls took minutes in lust before fx-parsehang (a parse-fuzz case). Each must be quick.
fn f<P: AsRef<std::path::Path>>(_p: P) {}
fn main() {
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    f(&target);
    match std::fs::read_link(&target) {
        Ok(_) => println!("ok"),
        Err(e) => println!("{:?} {}", e.kind(), e),
    }
}

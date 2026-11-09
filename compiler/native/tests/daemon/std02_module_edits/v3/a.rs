// Cell and sync overlays, plus a platform module.
use std::cell::OnceCell;
use std::io::Write;
use std::sync::LazyLock;

static N: LazyLock<u32> = LazyLock::new(|| 40 + 2);

pub fn go() {
    let c: OnceCell<u8> = OnceCell::new();
    let v = *c.get_or_init(|| 7);
    writeln!(std::io::stdout(), "{} {}", v, *N).unwrap();
}

// A `static LazyLock` whose closure is followed by a trailing comma (the
// rustfmt shape of a long initializer), and a method of the deref target
// that has the same name as LazyLock's associated fn `get`.
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Migration {
    tag: u8,
    name: String,
}

fn renamed(n: &str) -> Migration {
    Migration { tag: 1, name: n.to_string() }
}

pub static DB: LazyLock<HashMap<&'static str, Migration>> = LazyLock::new(
    || {
        let entries: Vec<(&'static str, Migration)> = vec![("a", renamed("x")), ("b", renamed("y"))];
        entries.into_iter().collect()
    },
);

static CODES: LazyLock<Vec<i32>> = LazyLock::new(|| vec![3, 4],);

static TYPED: LazyLock<Vec<u8>> = LazyLock::new(|| -> Vec<u8> { vec![9] },);

fn main() {
    println!("{:?}", DB.get("a"));
    println!("{:?}", DB.get("zz").map(|m| m.tag));
    println!("{} {:?} {:?}", DB.len(), *CODES, CODES.get(1));
    println!("{:?} {:?}", LazyLock::get(&TYPED), TYPED.get(0));
    println!("{:?}", LazyLock::get(&TYPED));
}

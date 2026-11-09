pub use base::*; // glob re-export: Thing, Kind, and the base_add macro ride through
pub use base::Thing;

pub fn via(v: i64) -> i64 {
    base::inner::get(&base::inner::make(v))
}

pub fn kind_val(k: Kind) -> i64 {
    match k {
        Kind::A => 1,
        Kind::B(x) => x,
    }
}

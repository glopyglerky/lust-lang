pub(crate) fn crated() -> i64 {
    3
}

pub(in crate::a) fn in_a() -> i64 {
    4
}

pub(super) fn for_inner() -> i64 {
    5
}

pub mod inner;

pub fn via_inner() -> i64 {
    inner::up()
}

// Sol's read-gap review (r06/t16), reversed: the import of a std trait goes away under an unchanged body.

struct S;
impl std::ops::Add for S { type Output = u8; fn add(self, _: S) -> u8 { 7 } }
fn get() -> u8 { S.add(S) }
fn main() { println!("{}", get()); }

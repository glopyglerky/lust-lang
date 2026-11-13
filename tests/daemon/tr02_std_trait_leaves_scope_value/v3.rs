// Sol's read-gap review (r06/t17), reversed: the import of a std trait goes away under an unchanged body.
use std::ops::Add;
struct S;
impl std::ops::Add for S { type Output = u8; fn add(self, _: S) -> u8 { 7 } }
trait Mine { fn add(&self, _: S) -> u8 { 9 } }
impl Mine for S {}
fn get() -> u8 { S.add(S) }
fn main() { println!("{}", get()); }

// #[track_caller] fns and std::panic::Location: caller(), file/line/column, Debug and Display, a generic and a method fn, propagation through a track_caller caller, a closure reporting its own location (W-CHECK-G3).
use std::panic::Location;
#[track_caller]
fn here() -> &'static Location<'static> { Location::caller() }
#[track_caller]
fn outer() -> u32 { here().line() }
fn plain() -> u32 { here().line() }
struct S;
impl S { #[track_caller] fn m(&self) -> u32 { Location::caller().column() } }
#[track_caller]
fn generic<T: std::fmt::Debug>(t: T) -> String { format!("{:?} {}", t, Location::caller()) }
fn main() {
    let l = here();
    println!("{} {} {}", l.file(), l.line(), l.column());
    println!("{:?}", l);
    println!("{}", l);
    println!("{} {}", outer(), plain());
    println!("{}", S.m());
    println!("{}", generic(5u8));
    let c = || here().line();
    println!("{}", c());
    println!("{}", Location::caller());
}

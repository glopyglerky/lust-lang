// Exhaustiveness reads a structural const pattern as the patterns of its value, as rustc does.
#![allow(dead_code)]
#[derive(PartialEq, Eq, Debug)] struct S(bool);
const T: S = S(true);
const F: S = S(false);
#[derive(PartialEq, Eq, Debug)] enum E { A, B(u8), C { x: bool } }
const B0: E = E::B(0);
const CT: E = E::C { x: true };
const CF: E = E::C { x: false };
const N: Option<(bool, bool)> = None;
const TT: Option<(bool, bool)> = Some((true, true));
const ARR: [bool; 2] = [true, false];
const SL: &[bool] = &[true];
const EMPTY: &[bool] = &[];
struct U;
fn s(x: S) -> u8 { match x { T => 1, F => 2 } }
fn e(x: E) -> u8 { match x { E::A => 0, B0 => 1, E::B(1..=255) => 2, CT => 3, CF => 4 } }
fn o(x: Option<(bool, bool)>) -> u8 { match x { N => 0, TT => 1, Some((false, _)) => 2, Some((true, false)) => 3 } }
fn a(x: [bool; 2]) -> u8 { match x { ARR => 1, [false, _] => 2, [true, true] => 3 } }
fn sl(x: &[bool]) -> u8 { match x { EMPTY => 0, SL => 1, [false] => 2, [_, _, ..] => 3 } }
fn r(x: &S) -> u8 { match x { &T => 1, &F => 2 } }
fn main() {
    println!("{} {}", s(S(true)), s(S(false)));
    println!("{} {} {} {} {}", e(E::A), e(E::B(0)), e(E::B(9)), e(E::C { x: true }), e(E::C { x: false }));
    println!("{} {} {} {}", o(None), o(Some((true, true))), o(Some((false, true))), o(Some((true, false))));
    println!("{} {} {}", a([true, false]), a([false, false]), a([true, true]));
    println!("{} {} {} {}", sl(&[]), sl(&[true]), sl(&[false]), sl(&[true, true]));
    println!("{} {}", r(&S(true)), r(&S(false)));
}

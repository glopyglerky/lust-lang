// derive(Default): structs, tuple and unit structs, enums with a #[default]
// unit variant (no bounds on the enum params, as rustc), generic structs
// (T: Default), Default::default() with the type from the context, and
// T::default() through a bound.
#[derive(Default, Debug)]
struct Inner { k: u16, c: char }
#[derive(Default, Debug)]
enum Kind { A(u8), #[default] B, C { x: u8 } }
#[derive(Default, Debug)]
struct Unit;
#[derive(Default, Debug)]
struct Tup(i8, String);
#[derive(Default, Debug)]
struct P { a: u32, b: bool, c: (u8, char), d: String, e: Vec<i32>, f: Option<u8>, g: f64, h: [u8; 3], i: [String; 0], j: Box<u64>, k: Inner, l: Kind, m: Unit, n: Tup, o: (), p: f32, q: i128, r: Option<Vec<String>> }
fn plain() {
    let p = P::default();
    println!("{:?}", p);
    let q: P = Default::default();
    println!("{} {:?} {:?}", q.a, q.l, q.n);
    let k: Kind = Default::default();
    println!("{:?} {:?}", k, Inner::default());
}
#[derive(Default, Debug)]
struct W<T> { t: T, v: Vec<T> }
#[derive(Default, Debug)]
enum GE<T> { #[default] A, B(T) }
struct NoDef;
#[derive(Debug, Default)]
struct GS { w: W<u8>, x: W<String> }
fn mk<T: Default>() -> T { T::default() }
fn generic() {
    let a: W<u8> = W::default();
    let b: W<String> = mk();
    let e: GE<NoDef> = GE::default();
    let s: GS = mk();
    println!("{:?} {:?} {} {:?}", a, b, matches!(e, GE::A), s);
}
#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum DE { A = 3, #[default] B = 7, C }
#[derive(Default, Debug, PartialEq)]
enum F { #[default] Only }
fn discs() { println!("{:?} {} {:?}", DE::default(), DE::default() as u8, F::default()); }
fn main() {
    plain();
    generic();
    discs();
}

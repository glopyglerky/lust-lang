// Niche values as rustc 1.99 stores them, seen through constant address
// equality: a constant shares storage with an equal-bytes constant of another
// type (4/8/16-byte constants merge by bytes, an uninit byte as 0).
// Before fx-niche lust printed other values on every line.
#![allow(dead_code)]
use std::ptr::addr_eq;

enum Y { A, B(char), C, D }
enum D7 { P(fn()), U0, U1, X(std::convert::Infallible) }
#[derive(Clone, Copy)]
enum G { A = 5, B = 6, C = 7 }
enum T { X, Y, Z(G) }

const OC: Option<char> = None;
const OOC: Option<Option<char>> = None;
const SOC: Option<Option<char>> = Some(None);
const OOB: Option<Option<bool>> = None;
const OOR: Option<Option<Result<u32, ()>>> = None;
const YC: Y = Y::C;
const YD: Y = Y::D;
const OD7: Option<D7> = None;
const TX: T = T::X;
const ONZ: Option<std::num::NonZeroU32> = None;
const ON128: Option<Option<(std::num::NonZeroU128, u8)>> = None;

fn main() {
    // One free value goes to the side nearer zero: -1 for char.
    println!("{} {}", addr_eq(&OC, &0xFFFF_FFFFu32), addr_eq(&OC, &0x110000u32));
    println!("{} {}", addr_eq(&OOC, &0xFFFF_FFFEu32), addr_eq(&SOC, &0xFFFF_FFFFu32));
    // bool's 0..=1 keeps the old rule (Option<bool> None is 2); one level up is 255.
    println!("{}", addr_eq(&OOB, &255u8));
    // Result<u32, ()> has a 4-byte tag (the unit field does not count).
    println!("{}", addr_eq(&OOR, &0xFFFF_FFFFu64));
    // Variants count by index across the dataful one.
    println!("{} {}", addr_eq(&YC, &0x110002u32), addr_eq(&YD, &0x110003u32));
    // An Infallible variant needs no tag value; the enum keeps 16 bytes.
    println!("{} {}", addr_eq(&OD7, &[u64::MAX, 0]), std::mem::size_of::<Option<D7>>());
    // A tag range 5..=7: two values below it.
    println!("{}", addr_eq(&TX, &3u8));
    println!("{}", addr_eq(&ONZ, &0u32));
    // A 16-byte tag: the uninit payload bytes keep it apart from an all-init array.
    println!("{} {}", addr_eq(&ON128, &[0u8; 48]), std::mem::size_of::<Option<Option<(std::num::NonZeroU128, u8)>>>());
}

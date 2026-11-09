// ARCH 4.4: scalar sizes, explicit discriminants, repr ints,
// repr(align), repr(transparent), repr(packed) size, ZSTs,
// fat pointers, tuples, arrays, MaybeUninit, and closure sizes.
#![allow(dead_code)]
use std::mem::{align_of, size_of, size_of_val};

#[repr(u8)]
enum RU8 {
    A,
    B = 200,
}
#[repr(i32)]
enum RI32 {
    A = -5,
    B,
}
#[repr(u64)]
enum RU64 {
    A = 0,
    B = 1,
    C = 2,
}
#[repr(usize)]
enum RUsize {
    A,
    B,
}
enum NegDisc {
    A = -3,
    B = 100,
}
#[repr(transparent)]
struct Wrap(u32);
#[repr(align(16))]
struct Aligned {
    a: u8,
}
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,
}
struct Zst;
struct WithPhantom<T> {
    marker: std::marker::PhantomData<T>,
    n: u8,
}

fn p<T>(name: &str) {
    println!("{name} size={} align={}", size_of::<T>(), align_of::<T>());
}
fn main() {
    p::<bool>("bool");
    p::<char>("char");
    p::<u8>("u8");
    p::<i16>("i16");
    p::<u32>("u32");
    p::<i64>("i64");
    p::<u128>("u128");
    p::<i128>("i128");
    p::<f32>("f32");
    p::<f64>("f64");
    p::<usize>("usize");
    p::<*const u8>("raw_const");
    p::<*mut u8>("raw_mut");
    p::<&'static u8>("ref");
    p::<&'static [u8]>("ref_slice");
    p::<&'static str>("ref_str");
    p::<Box<dyn std::fmt::Debug>>("box_dyn");
    p::<fn(u8) -> u8>("fn_ptr");
    p::<Option<fn(u8) -> u8>>("opt_fn_ptr");
    p::<Option<std::num::NonZeroU32>>("opt_nonzero");
    p::<(u8, u32)>("tuple_u8_u32");
    p::<(u64, u8)>("tuple_u64_u8");
    p::<[u8; 3]>("arr_u8_3");
    p::<[u32; 2]>("arr_u32_2");
    p::<String>("String");
    p::<Vec<u8>>("Vec_u8");
    p::<RU8>("repr_u8_enum");
    p::<RI32>("repr_i32_enum");
    p::<RU64>("repr_u64_enum");
    p::<RUsize>("repr_usize_enum");
    p::<NegDisc>("enum_neg_disc");
    p::<Wrap>("transparent_u32");
    p::<Aligned>("align16_u8");
    p::<Packed>("packed_u8_u32");
    p::<Zst>("zst");
    p::<WithPhantom<u64>>("phantom_u8");
    p::<std::mem::MaybeUninit<u64>>("maybeuninit_u64");
    p::<Option<char>>("opt_char");
    println!("disc {} {} {} {}", RU8::B as u8, RI32::A as i32, NegDisc::A as i32, NegDisc::B as i32);
    // Packed field read by value is allowed (no reference is made).
    let pk = Packed { a: 1, b: 7 };
    let copied_b = pk.b;
    println!("packed_read {} {}", pk.a, copied_b);
    // Closure env layout, from the probe r2-closure-layout.rs.
    let empty = || {};
    let value = 3u8;
    let captured = move || value;
    println!("closure {} {}", size_of_val(&empty), size_of_val(&captured));
}

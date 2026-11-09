// Promoted constants whose value release codegen computes at compile time
// (cgen/kconst.rs eval_promoted): integer arithmetic wraps to the type's
// width, shifts drop bits, float to int casts saturate, float casts keep
// their source width, and casts and compares see signedness. Equal
// promoted values share one address inside a fn, also in a loop and
// across fields read from aggregates, and a promoted value that holds
// strs points at the canonical literal.
use std::ptr;

#[derive(Clone, Copy, PartialEq, Debug)]
enum E {
    A,
    B(u8, i16),
}

fn shifts() -> (&'static u8, &'static i8, &'static u32, &'static i64) {
    (&(0xFFu8 << 4), &(-128i8 >> 3), &(0x8000_0000u32 >> 31), &(-1i64 << 40))
}

fn casts() -> [&'static u64; 4] {
    [&(-1i32 as u64), &(-1i8 as u8 as u64), &(300u16 as u8 as u64), &(u32::MAX as i32 as i64 as u64)]
}

fn floats() -> (&'static u8, &'static i8, &'static u32, &'static i32, &'static u64) {
    (&(300.7f64 as u8), &(-200.5f32 as i8), &(-1.5f32 as u32), &(f64::NAN as i32), &(1e30f64 as u64))
}

fn compares() -> [&'static bool; 6] {
    [&((200u8 as i8) < 0), &(u64::MAX > 1), &(-1i32 < 1), &((-1i64 as u64) > 5), &('a' < 'b'), &(1.5f64 > 1.25)]
}

fn widths() -> (&'static f64, &'static f32, &'static u8) {
    (&(1.1f32 as f64), &(0.1f64 as f32), &((2.9f64 as f32) as u8))
}

fn main() {
    println!("{:?}", shifts());
    println!("{:?}", casts());
    println!("{:?}", floats());
    println!("{:?}", compares());
    println!("{:?}", widths());
    let a: &(u8, E, [i16; 3]) = &(7, E::B(3, -4), [1, -2, 3]);
    let b: &(u8, E, [i16; 3]) = &(7, E::B(3, -4), [1, -2, 3]);
    println!("{:?} {}", a, ptr::eq(a, b));
    println!("{} {}", ptr::eq(&E::A, &E::A), ptr::eq(&(1u32 + 2), &3u32));
    println!("{} {}", ptr::addr_eq(&5i32, &5u32), ptr::addr_eq(&[1u8, 0, 0, 0], &1u32));
    let mut v = Vec::new();
    for i in 0..3u32 {
        let r: &u32 = &(10 * 4 + 2);
        v.push(r as *const u32);
        assert_eq!(*r, 42 + i - i);
    }
    println!("loop {}", v[0] == v[1] && v[1] == v[2]);
    let p: &(&str, u8) = &("x", 1);
    let q: &Option<&str> = &Some("yz");
    let r: &[&str; 2] = &["x", "yz"];
    println!("{:?} {:?} {:?}", p, q, r);
    println!("{} {} {}", p.0.as_ptr() == "x".as_ptr(), q.unwrap().as_ptr() == r[1].as_ptr(), ptr::eq(&"x", &"x"));
    println!("{} {}", ptr::eq(p, &("x", 1)), ptr::eq(r, &["x", "yz"]));
    let s: &[i16] = &[-1, 2, -3];
    let t: &[i16; 3] = &[-1, 2, -3];
    println!("{:?} {}", s, s.as_ptr() == t.as_ptr());
}

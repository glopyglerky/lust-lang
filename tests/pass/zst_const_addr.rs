// A zero-sized constant has no storage in rustc: its address is its
// alignment. `{:p}` of `&()` prints 0x1, of `&[0u64; 0]` 0x8, and
// `"".as_ptr()` is 0x1, in promoted temps, consts, statics, and slices.
use std::ptr;

#[derive(Debug, Clone, Copy)]
struct No;
#[derive(Debug, Clone, Copy)]
#[repr(align(8))]
struct A8;
#[derive(Debug, Clone, Copy)]
struct Two(No, [u32; 0]);

const C: &A8 = &A8;
const E: &[u16] = &[];
static S: &str = "";

fn unit() -> &'static () {
    &()
}

fn two() -> &'static Two {
    &Two(No, [])
}

fn main() {
    println!("{:p} {:p} {:p}", &(), &No, &A8);
    println!("{:p} {:p}", &[0u16; 0], &[0u64; 0]);
    println!("{:p} {:p}", unit(), two());
    println!("{:p} {:p} {:p}", C, E.as_ptr(), S.as_ptr());
    println!("{:p} {:p}", "".as_ptr(), b"".as_ptr());
    let s: &[u64] = &[];
    println!("{:p} {}", s.as_ptr(), s.len());
    for x in &[(), (), ()] {
        println!("{}", x as *const () as usize);
    }
    println!("{}", ptr::eq(unit(), &()));
    println!("{}", (&No as *const No as usize) == (&A8 as *const A8 as usize));
    let local = ();
    println!("{}", ptr::eq(&local, &()));
}

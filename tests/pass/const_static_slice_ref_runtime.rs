// A const holding a reference to a static, used at run time, is the
// static's address: at a slice type it is the fat pointer over the static
// array (this was a tcc compile error), and a const reference into a static
// points at the static's element.
static S: [u32; 3] = [5, 6, 7];
const WHOLE: &[u32] = &S;
const ARRAY: &[u32; 3] = &S;
const ELEM: &u32 = &S[2];
const SUB: &[u32] = &T.1;
static T: (u8, [u32; 2]) = (1, [8, 9]);
fn main() {
    println!("{} {}", WHOLE.len(), std::ptr::eq(WHOLE.as_ptr(), S.as_ptr()));
    println!("{}", std::ptr::eq(ARRAY, &S));
    println!("{} {}", ELEM, std::ptr::eq(ELEM, &S[2]));
    println!("{:?} {}", SUB, std::ptr::eq(SUB.as_ptr(), T.1.as_ptr()));
    let sum: u32 = WHOLE.iter().sum();
    println!("{sum}");
}

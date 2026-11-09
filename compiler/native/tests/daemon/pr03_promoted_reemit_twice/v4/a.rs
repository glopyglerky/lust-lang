#[derive(Debug)]
pub struct P {
    pub a: u8,
    pub b: u16,
}
const C: u16 = 9;

pub fn pair(it: Vec<u32>) -> (Option<u32>, &'static u8) {
    (it.into_iter().next(), &7)
}
pub fn sums(v: Vec<u32>) -> (u32, &'static P, &'static [u16]) {
    (v.iter().sum(), &P { a: 3, b: 4 }, &[C, 2])
}
pub fn forl(v: Vec<u32>) -> (u32, &'static (u8, char), &'static Option<String>) {
    let mut s = 0;
    for x in v.iter() {
        s += x;
    }
    (s, &(1, 'z'), &None)
}

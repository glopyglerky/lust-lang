// A `try_into` result stored before a later annotation fixes the
// target: the TryInto Error projection normalizes once it is known.
#[derive(Debug)]
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let x = 300u32;
    let r = x.try_into();
    let r: Result<u8, std::num::TryFromIntError> = r;
    println!("{:?}", r);
    let s: &[u8] = &[1, 2];
    let q = s.try_into();
    let q: Result<[u8; 2], std::array::TryFromSliceError> = q;
    println!("{:?}", q);
    let v = vec![D(1), D(2), D(3)];
    let r = v.try_into();
    let a: [D; 2] = match r {
        Ok(a) => a,
        Err(v) => {
            let v: Vec<D> = v;
            println!("back {}", v.len());
            [D(7), D(8)]
        }
    };
    println!("{:?}", a);
}

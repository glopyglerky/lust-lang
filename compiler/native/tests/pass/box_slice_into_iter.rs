// A Box<[T]> iterates by value in Rust 2024: for loops and into_iter
// yield T through vec::IntoIter, dropping what is left on break.
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let b: Box<[String]> = vec![String::from("a"), String::from("b")].into_boxed_slice();
    for x in b {
        println!("{}", x);
    }
    let c: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    let v: Vec<i32> = c.into_iter().map(|x| x * 2).collect();
    println!("{:?}", v);
    let n: Box<[Noisy]> = vec![Noisy(1), Noisy(2), Noisy(3)].into_boxed_slice();
    for x in n {
        if x.0 == 2 {
            println!("break at {}", x.0);
            break;
        }
    }
    let z: Box<[()]> = vec![(); 4].into_boxed_slice();
    println!("{}", z.into_iter().count());
    let u: Box<[()]> = vec![(); 2].into_boxed_slice();
    println!("{}", u.into_vec().capacity() == usize::MAX);
}

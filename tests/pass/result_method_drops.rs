// Result::ok, err, and unwrap_or drop the side they do not return.
struct Noisy(u8);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let a: Result<Noisy, Noisy> = Err(Noisy(1));
    println!("{:?}", a.ok().is_none());
    let b: Result<Noisy, Noisy> = Ok(Noisy(2));
    println!("{:?}", b.err().is_none());
    let c: Result<Noisy, Noisy> = Ok(Noisy(3));
    let k = c.unwrap_or(Noisy(4));
    println!("kept {}", k.0);
    let d: Result<Noisy, Noisy> = Err(Noisy(5));
    let k2 = d.unwrap_or(Noisy(6));
    println!("kept {}", k2.0);
}

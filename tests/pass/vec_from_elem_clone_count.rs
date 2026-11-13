// `vec![x; n]` and `resize` call a written Clone n-1 times and move the value
// into the last slot; n == 0 drops the value. A zero-sized element with a
// written Clone or Drop takes no fast path, so every call still runs.
struct Z;
impl Clone for Z {
    fn clone(&self) -> Z {
        println!("clone Z");
        Z
    }
}
impl Drop for Z {
    fn drop(&mut self) {
        println!("drop Z");
    }
}

struct B(u32);
impl Clone for B {
    fn clone(&self) -> B {
        println!("clone B{}", self.0);
        B(self.0 + 1)
    }
}

fn main() {
    for n in [0usize, 1, 3] {
        println!("n={n}");
        let v = vec![Z; n];
        println!("len {}", v.len());
    }
    let v = vec![B(0); 3];
    println!("{:?}", v.iter().map(|b| b.0).collect::<Vec<_>>());
    let mut w: Vec<Z> = Vec::new();
    w.resize(2, Z);
    println!("resized {}", w.len());
    w.resize(1, Z);
    println!("end");
}

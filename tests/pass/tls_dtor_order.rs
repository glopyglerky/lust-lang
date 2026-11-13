// Thread-local destructors run at the main thread's exit, after main's own
// locals, newest registration first (a value registers at its first access,
// a const value too). A destructor may read a key that is still alive, and a
// key first touched during destruction is made and dropped in turn.
// try_with on a destroyed key is Err.
struct D(&'static str, u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
        if self.1 == 1 {
            B.with(|b| println!("a's drop sees {}", b.0));
            C.with(|c| println!("a's drop made {}", c.0));
        }
        if self.1 == 3 {
            match A.try_with(|a| a.1) {
                Ok(n) => println!("c's drop: a alive {}", n),
                Err(_) => println!("c's drop: a destroyed"),
            }
        }
    }
}

thread_local! {
    static A: D = D("a", 1);
    static B: D = const { D("b", 2) };
    static C: D = D("c", 3);
    static N: u32 = 5;
}

fn main() {
    let _local = D("main local", 0);
    B.with(|b| println!("touch {}", b.0));
    N.with(|n| println!("touch {}", n));
    A.with(|a| println!("touch {}", a.0));
    println!("end of main");
}

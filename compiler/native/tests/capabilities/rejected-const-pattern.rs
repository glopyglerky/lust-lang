// capability: rejected: non-scalar const patterns
// expect: L9001

static S: u32 = 1;
const R: &u32 = &S;

fn main() {
    match &1u32 {
        R => println!("one"),
        _ => println!("other"),
    }
}

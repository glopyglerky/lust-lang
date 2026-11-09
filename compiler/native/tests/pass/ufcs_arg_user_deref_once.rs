// A trait path call whose receiver argument derefs through a user
// `Deref` (`Count::count_it(&**d)`, `fmt::Debug::fmt(&**self, f)`):
// check trial-checks that argument for the Self type, then checks it
// again; the trial's place step must not stay, or `deref` runs twice.
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, Copy)]
struct Node(u32);

struct Decls(Vec<Node>);

impl Deref for Decls {
    type Target = [Node];
    fn deref(&self) -> &[Node] {
        println!("deref");
        &self.0
    }
}

impl fmt::Debug for Decls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

trait Count {
    fn count_it(&self) -> usize;
}

impl Count for [Node] {
    fn count_it(&self) -> usize {
        self.len()
    }
}

fn main() {
    let d = Decls(vec![Node(1), Node(2)]);
    println!("{}", Count::count_it(&**&d));
    println!("{:?}", d);
}

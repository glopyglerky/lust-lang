// expect: L0730
// rustc E0320: the enum recurses with an array argument through Rc.
enum Nest<T> { Leaf(T), Node(std::rc::Rc<Nest<[T; 2]>>) }
fn depth(n: &Nest<u8>) -> u8 { match n { Nest::Leaf(v) => *v, Nest::Node(_) => 0 } }
fn main() { let n = Nest::Leaf(4u8); println!("{}", depth(&n)); }

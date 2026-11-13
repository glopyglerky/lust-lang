// Pattern constant indexes name disjoint array elements: two ref mut
// bindings into different slots are fine, as rustc's ConstantIndex.
#[derive(Debug)]
enum N { V(Vec<u8>), Z }
fn m(v: &mut Option<[N; 4]>) {
    match v {
        Some([N::V(_), N::V(a), .., N::V(_), N::V(b)]) => println!("{:?} {:?}", a, b),
        _ => {}
    }
}
fn main() { let mut o = Some([N::V(vec![1]), N::V(vec![2]), N::Z, N::V(vec![4])]); m(&mut o); }

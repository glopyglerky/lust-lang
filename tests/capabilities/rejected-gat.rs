// capability: rejected: generic associated types and associated type defaults
// expect: L0621

trait L {
    type I<'a>
    where
        Self: 'a;
    fn get<'a>(&'a self) -> Self::I<'a>;
}

struct V(Vec<u8>);

impl L for V {
    type I<'a> = &'a [u8];
    fn get<'a>(&'a self) -> &'a [u8] {
        &self.0
    }
}

fn main() {
    println!("{:?}", V(vec![1]).get());
}

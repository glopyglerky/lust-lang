// A body `use Enum as V;` binds the enum under the new name (ts_goport
// `use ParsingContext as PC;`, `use CompilerOptionsValue as V;`), a body
// static's type names a block alias (`type Garbage = Box<dyn Send>;`),
// and Self names the Adt in struct and enum field types.
use std::sync::Mutex;
mod m {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ParsingContext { SourceElements, BlockStatements(u8) }
    #[derive(Debug)]
    pub enum Value { Nil, Int(i32) }
    pub fn seven() -> u8 { 7 }
}
use m::*;

fn is_list_terminator(c: ParsingContext) -> u8 {
    use ParsingContext as PC;
    match c {
        PC::SourceElements => 0,
        PC::BlockStatements(x) => x,
    }
}

fn values() -> Vec<Value> {
    use Value as V;
    use m::seven as s7;
    vec![V::Nil, V::Int(s7() as i32)]
}

fn drop_in_background<T: Send + 'static>(value: T) -> bool {
    type Garbage = Box<dyn Send>;
    static QUEUE: std::sync::OnceLock<Option<Mutex<Vec<Garbage>>>> = std::sync::OnceLock::new();
    let queue = QUEUE.get_or_init(|| Some(Mutex::new(Vec::new())));
    let value: Garbage = Box::new(value);
    match queue {
        Some(q) => {
            q.lock().unwrap().push(value);
            true
        }
        None => false,
    }
}

#[derive(Debug)]
enum List { Cons(i32, Box<Self>), Nil }

fn main() {
    println!("{} {}", is_list_terminator(ParsingContext::SourceElements), is_list_terminator(ParsingContext::BlockStatements(4)));
    println!("{:?}", values());
    println!("{}", drop_in_background(vec![1u8]));
    #[derive(Debug, Default, Clone, PartialEq)]
    struct Tree { kids: Vec<Self>, n: u8 }
    let t = Tree { kids: vec![Tree::default()], n: 3 };
    println!("{:?} {}", t.clone(), t == t);
    println!("{:?}", List::Cons(1, Box::new(List::Nil)));
}

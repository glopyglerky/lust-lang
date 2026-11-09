// Items in fn bodies resolve as rustc does (tf-bodyitems): a block
// type item shadows a module item of its name (ts_goport's State,
// Kind, Mode, MemberInfo), variant paths and `use` of a block enum's
// variants (before or after the declaration), unit and tuple struct
// constructors, derived trait fns by path, and `use Enum::Variant as V`
// of a module enum inside a closure (ts_goport enum_maps.rs).
use std::sync::LazyLock;

mod other {
    #[derive(Debug)]
    pub enum State { Q, R }
    #[derive(Debug)]
    pub struct MemberInfo { pub z: u8 }
    pub fn f() -> (State, State, MemberInfo) { (State::Q, State::R, MemberInfo { z: 3 }) }
}

mod opts {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Value { Nil, Num(i32), Str(String) }
}
use opts::*;

#[allow(dead_code)]
#[derive(Debug)]
enum Kind { Root }

static MAP: LazyLock<Vec<Value>> = LazyLock::new(|| {
    use Value::Num as V;
    vec![V(1), V(2), Value::Nil]
});

struct H;
impl H {
    fn go(&self) -> usize {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum State { NotTaken, Waiting, Done }
        let mut states = vec![State::NotTaken; 3];
        states[1] = State::Waiting;
        states[2] = State::Done;
        let s: State = states[0];
        println!("{:?} {:?}", s, states);
        states.iter().filter(|s| **s != State::NotTaken).count()
    }
}

fn kinds() {
    let k = Kind::Interface(Box::new(5));
    enum Kind { Interface(Box<i32>), Reference }
    let t = match k { Kind::Interface(b) => *b, Kind::Reference => 0 };
    let r = Kind::Reference;
    println!("{} {}", t, matches!(r, Kind::Reference));
}

fn main() {
    println!("{}", H.go());
    kinds();
    #[derive(Clone, Default, Debug)]
    struct MemberInfo { missed: Vec<String>, name: String }
    let mut m: std::collections::BTreeMap<u32, MemberInfo> = std::collections::BTreeMap::new();
    m.insert(1, MemberInfo::default());
    let x: &mut MemberInfo = m.get_mut(&1).unwrap();
    x.name.push('z');
    x.missed.push("p".to_string());
    println!("{:?}", m[&1]);
    println!("{:?}", other::f());
    println!("{:?}", *MAP);
    {
        use Mode::*;
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum Mode { None, Modifiers, Decorators }
        let mut last = None;
        for n in [Modifiers, Decorators, Decorators] {
            if n != last { println!("{:?} -> {:?}", last, n); }
            last = n;
        }
        use Mode::Decorators as D;
        match last { D => println!("d"), None | Modifiers => println!("other") }
    }
    let c = || {
        use Value::Str as S;
        S("s".to_string())
    };
    println!("{:?}", c());
    #[derive(Debug)]
    struct Unit;
    #[derive(Debug)]
    struct Pair(u8, u8);
    let mk = Pair;
    println!("{:?} {:?} {:?}", Unit, Pair(1, 2), mk(3, 4));
    println!("{:?}", Kind::Root);
    aliases();
}

// Type aliases in bodies, block struct field types from other modules,
// and a block type item that shadows builtin and module names.
mod more {
    #[derive(Debug, Default, Clone)]
    pub struct Thing(pub u8);
}
use more::Thing;
#[allow(dead_code)]
type Count = u64;
pub fn aliases() {
    type V = Vec<A>;
    type A = (u8, Thing);
    #[derive(Debug, Default)]
    struct Holder { a: Thing, b: more::Thing, c: Vec<crate::more::Thing> }
    let v: V = vec![(1, Thing(2))];
    type Count = u8;
    let c: Count = 255;
    {
        type Count = i8;
        let d: Count = -1;
        #[derive(Debug)]
        struct Thing(i32);
        println!("{} {:?}", d, Thing(4));
    }
    println!("{:?} {} {:?} {}", v, c, Holder::default(), std::mem::size_of::<Count>());
}

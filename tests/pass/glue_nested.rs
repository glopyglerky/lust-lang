// Glue through nested types (release C units copy nested glue too, `cgen::fn_unit_parts`):
// derived Clone, PartialEq, Eq, Hash, Default and Debug, and drop glue, of structs, tuple
// structs and enums whose fields hold other such types inside Vec, Option, Box, Rc, tuples and
// arrays. Each value is cloned, compared, hashed into a set and a map, defaulted, printed, moved
// into a Vec and dropped; every D drop prints.
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D({})", self.0);
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct Leaf {
    id: u8,
    tag: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct Mid {
    leaf: Leaf,
    leaves: Vec<Leaf>,
    boxed: Box<Leaf>,
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct Top(Mid, Option<Rc<Mid>>, [Leaf; 2], (bool, Leaf));

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
enum Node {
    #[default]
    Empty,
    One(Top),
    Many(Vec<Node>, D),
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct Owner {
    d: D,
    node: Node,
    kids: Vec<(D, Option<Box<D>>)>,
}

fn leaf(id: u8) -> Leaf {
    Leaf { id, tag: if id % 2 == 0 { Some(format!("t{id}")) } else { None } }
}

fn mid(id: u8) -> Mid {
    Mid { leaf: leaf(id), leaves: vec![leaf(id + 1), leaf(id + 2)], boxed: Box::new(leaf(id + 3)) }
}

fn top(id: u8) -> Top {
    Top(mid(id), if id % 3 == 0 { None } else { Some(Rc::new(mid(id + 10))) }, [leaf(id + 20), leaf(id + 21)], (id % 2 == 1, leaf(id + 30)))
}

fn owner(n: i32) -> Owner {
    let node = if n % 2 == 0 { Node::One(top(n as u8)) } else { Node::Many(vec![Node::Empty, Node::One(top(n as u8 + 1))], D(n * 100)) };
    Owner { d: D(n), node, kids: vec![(D(n * 10), None), (D(n * 10 + 1), Some(Box::new(D(n * 10 + 2))))] }
}

fn main() {
    let a = owner(1);
    let b = a.clone();
    println!("eq {} {}", a == b, a == owner(2));
    println!("{:?}", b.node);
    let d: Owner = Default::default();
    println!("default {:?} kids {}", d.node, d.kids.len());
    let mut set = HashSet::new();
    set.insert(top(4));
    set.insert(top(4).clone());
    set.insert(top(5));
    println!("set {} {} {}", set.len(), set.contains(&top(4)), set.contains(&top(6)));
    let mut map: HashMap<Node, i32> = HashMap::new();
    *map.entry(Node::One(top(7))).or_insert(0) += 2;
    *map.entry(Node::One(top(7))).or_insert(0) += 3;
    map.insert(Node::Empty, 9);
    println!("map {} {:?} {:?}", map.len(), map.get(&Node::One(top(7))), map.get(&Node::Empty));
    let mut owners = vec![b, owner(3)];
    owners.push(a);
    owners.truncate(2);
    println!("owners {}", owners.len());
    let mut slot = Some(owner(4));
    slot = None;
    println!("slot {}", slot.is_some());
    drop(owners);
    println!("end");
}

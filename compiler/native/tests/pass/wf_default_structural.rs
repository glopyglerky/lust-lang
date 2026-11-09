// A written type whose definition bounds a parameter by `Default` takes core's
// structural impls: arrays (N <= 32, any element at N = 0), tuples of at most
// 12 elements, `&str` and `&[T]`. ts-rust: `LinkStore<SymbolId, [SymbolTable; 2]>`.
use std::collections::HashMap;

struct Store<V: Default> {
    items: Vec<V>,
}

impl<V: Default> Store<V> {
    fn add(&mut self) -> &mut V {
        self.items.push(V::default());
        self.items.last_mut().unwrap()
    }
}

struct NoDefault;

struct Checker {
    links: Store<[HashMap<u32, u32>; 2]>,
    empty: Store<[NoDefault; 0]>,
    full: Store<[u8; 32]>,
    nested: Store<[[String; 2]; 3]>,
    pair: Store<(u8, String)>,
    twelve: Store<(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)>,
    text: Store<&'static str>,
    bytes: Store<&'static [u8]>,
    mixed: Store<[(u8, [bool; 2]); 4]>,
}

fn main() {
    let mut c = Checker {
        links: Store { items: Vec::new() },
        empty: Store { items: Vec::new() },
        full: Store { items: Vec::new() },
        nested: Store { items: Vec::new() },
        pair: Store { items: Vec::new() },
        twelve: Store { items: Vec::new() },
        text: Store { items: Vec::new() },
        bytes: Store { items: Vec::new() },
        mixed: Store { items: Vec::new() },
    };
    c.links.add()[1].insert(1, 2);
    c.full.add()[31] = 7;
    c.nested.add()[2][1].push_str("x");
    c.pair.add().1.push('p');
    c.twelve.add().11 = 12;
    println!("{:?}", c.links.items);
    println!("{} {}", c.empty.items.len(), c.full.items[0].iter().map(|&b| b as u32).sum::<u32>());
    println!("{:?} {:?}", c.nested.items, c.pair.items);
    println!("{:?}", c.twelve.items);
    println!("{:?} {:?}", c.text.add(), c.bytes.add());
    println!("{:?}", c.mixed.add());
}

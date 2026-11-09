fn apply<F: Fn(u8) -> u16>(f: F, x: u8) -> u16 {
    f(x) + 1
}

fn twice<F: Fn(u16) -> u16>(f: F) -> u16 {
    let mut s: u16 = 0;
    let mut i = 0;
    while i < 3 {
        s += f(i as u16);
        i += 1;
    }
    s
}

struct Bag<T> {
    items: [Option<T>; 2],
    len: usize,
}

impl<T: Copy> Bag<T> {
    fn new() -> Self {
        Bag { items: [None, None], len: 0 }
    }
    fn push(&mut self, value: T) {
        self.items[self.len] = Some(value);
        self.len += 1;
    }
    fn map<U: Copy, F: Fn(T) -> U>(&self, f: F) -> Bag<U> {
        let mut out = Bag::new();
        let mut i = 0;
        while i < self.len {
            out.push(f(self.items[i].unwrap()));
            i += 1;
        }
        out
    }
}

fn main() {
    println!("{}", apply(|x: u8| x as u16 * 2, 3));
    println!("{}", twice(|x| x + 10));
    let mut bag: Bag<u8> = Bag::new();
    bag.push(3);
    bag.push(4);
    let wide: Bag<u16> = bag.map(|x| x as u16 * 10);
    println!("{} {}", wide.items[0].unwrap(), wide.items[1].unwrap());
}

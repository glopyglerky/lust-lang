// The generic half of a realish module (bench/realish/make_generic.py):
// user traits with a default method, generic structs with a generic
// impl, trait impls, generic fns, and closures (capturing and not)
// passed to generic fns. make_generic.py appends one renamed copy of
// this file to each warehouse module, so the crate mixes plain and
// generic code the way ts-rust does (see the density note in
// make_generic.py).

trait Scored {
    fn score(&self) -> i64;
    fn label(&self) -> String {
        format!("s{}", self.score())
    }
}

trait Merge {
    fn merge(&self, other: &Self) -> Self;
}

#[derive(Debug, Clone)]
struct Pair<A, B> {
    a: A,
    b: B,
}

#[derive(Debug, Clone)]
struct Ledger<T> {
    items: Vec<T>,
    total: i64,
}

impl<T: Scored + Clone> Ledger<T> {
    fn new() -> Ledger<T> {
        Ledger { items: Vec::new(), total: 0 }
    }

    fn push(&mut self, t: T) {
        self.total += t.score();
        self.items.push(t);
    }

    fn best(&self) -> Option<T> {
        let mut best: Option<T> = None;
        let mut i = 0;
        while i < self.items.len() {
            let better = match &best {
                Some(b) => self.items[i].score() > b.score(),
                None => true,
            };
            if better {
                best = Some(self.items[i].clone());
            }
            i += 1;
        }
        best
    }

    fn count_where<F: Fn(i64) -> bool>(&self, f: F) -> usize {
        let mut n = 0;
        let mut i = 0;
        while i < self.items.len() {
            if f(self.items[i].score()) {
                n += 1;
            }
            i += 1;
        }
        n
    }
}

#[derive(Debug, Clone)]
struct Widget {
    id: u32,
    weight: i64,
}

#[derive(Debug, Clone)]
struct Gadget {
    id: u32,
    power: i64,
    cost: i64,
}

impl Scored for Widget {
    fn score(&self) -> i64 {
        self.weight * 2 + self.id as i64
    }
}

impl Scored for Gadget {
    fn score(&self) -> i64 {
        self.power * 3 - self.cost
    }
    fn label(&self) -> String {
        format!("g{}", self.id)
    }
}

impl Merge for Widget {
    fn merge(&self, other: &Widget) -> Widget {
        Widget { id: self.id.max(other.id), weight: self.weight + other.weight }
    }
}

fn sum_scores<T: Scored>(xs: &Vec<T>) -> i64 {
    let mut s = 0;
    let mut i = 0;
    while i < xs.len() {
        s += xs[i].score();
        i += 1;
    }
    s
}

fn merge_all<T: Merge + Clone>(xs: &Vec<T>) -> Option<T> {
    if xs.is_empty() {
        return None;
    }
    let mut acc = xs[0].clone();
    let mut i = 1;
    while i < xs.len() {
        acc = acc.merge(&xs[i]);
        i += 1;
    }
    Some(acc)
}

fn pair_score<A: Scored, B: Scored>(p: &Pair<A, B>) -> i64 {
    p.a.score() * 10 + p.b.score()
}

fn apply_twice<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
    f(f(x))
}

fn closure_mix(seed: i64) -> i64 {
    let inc = |x: i64| x + 1;
    let dbl = |x: i64| x * 2;
    let a = apply_twice(|x| x - 3, seed);
    let b = apply_twice(|x| x % 97 + 5, a);
    let c = apply_twice(inc, b) + apply_twice(dbl, 3);
    let d = apply_twice(|x| if x > 50 { x / 2 } else { x + 11 }, c);
    // Capturing closures: by reference into a generic fn, and an FnMut
    // that counts.
    let step = seed % 7 + 1;
    let shift = |x: i64| x + step;
    let scale = |x: i64| x * step % 1009;
    let e = apply_twice(shift, d) + apply_twice(scale, b);
    let f = apply_twice(|x| x - step + a % 5, e);
    let mut calls = 0;
    let mut note = |x: i64| {
        calls += 1;
        x % 13
    };
    let g = note(e) + note(f) + note(c);
    inc(d) + dbl(a) + f + g + calls
}

fn run_generic() -> i64 {
    let mut l: Ledger<Widget> = Ledger::new();
    let mut g: Ledger<Gadget> = Ledger::new();
    let mut k = 0;
    while k < 20 {
        l.push(Widget { id: k, weight: (k as i64 * 7) % 13 });
        g.push(Gadget { id: k, power: (k as i64 * 5) % 11, cost: k as i64 % 4 });
        k += 1;
    }
    let big = l.count_where(|s| s > 10);
    let small = g.count_where(|s| s < 5);
    let best = match l.best() {
        Some(w) => w.score(),
        None => 0,
    };
    let label = match g.best() {
        Some(x) => x.label(),
        None => String::new(),
    };
    let merged = match merge_all(&l.items) {
        Some(w) => w.weight,
        None => 0,
    };
    let p = Pair { a: Widget { id: 1, weight: 2 }, b: Gadget { id: 3, power: 4, cost: 1 } };
    let t = apply_twice(|x| x * 3 + 1, best);
    let mix = closure_mix(best);
    let total = mix + sum_scores(&l.items) + sum_scores(&g.items) + pair_score(&p) + t + merged + big as i64 + small as i64;
    total + label.len() as i64 + l.items[0].label().len() as i64
}

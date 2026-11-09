// capability: patterns: _ wildcard
// capability: patterns: bindings (mut, ref, default binding modes)
// capability: patterns: literals
// capability: patterns: paths and unit variants
// capability: patterns: tuple-struct patterns
// capability: patterns: tuple patterns with ..
// capability: patterns: struct patterns with ..
// capability: patterns: or patterns a | b
// capability: patterns: range patterns
// capability: patterns: reference patterns &p and &mut p
// capability: patterns: @ bindings
// capability: patterns: slice patterns [a, rest @ .., b]
// capability: patterns: half-open range patterns
// expect: ok

#[derive(Debug)]
enum Op {
    Nop,
    Push(u32),
}

struct Pt {
    x: i32,
    y: i32,
    z: i32,
}

fn main() {
    let ops = [Op::Nop, Op::Push(3)];
    let mut acc = 0;
    for op in &ops {
        match op {
            Op::Nop => {}
            Op::Push(n) => acc += n,
        }
    }
    let (first, ..) = (1, 2, 3);
    let Pt { x, .. } = Pt { x: 4, y: 5, z: 6 };
    let mut owned = String::from("m");
    let ref r = owned;
    let rl = r.len();
    let mut m = 2;
    m += 1;
    let &a = &7;
    let &mut b = &mut 8;
    let v = [1, 2, 3, 4];
    let mid = match v {
        [h, rest @ .., t] => h + t + rest.len() as i32,
    };
    let kind = match acc {
        0 => "zero",
        n @ 1..=5 => if n == 3 { "three" } else { "small" },
        6.. => "big",
    };
    let c = match 'q' {
        'a' | 'e' => 1,
        'p'..='r' => 2,
        _ => 3,
    };
    owned.push('!');
    let p = Pt { x: 1, y: 2, z: 3 };
    println!("{acc} {first} {x} {rl} {m} {a} {b} {mid} {kind} {c} {owned} {}", p.y + p.z);
}

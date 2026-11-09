// Layout-driven codegen: niche-filled and reordered aggregates must
// behave (and drop, clone, compare, and print) exactly as in rustc.
#[derive(Debug, PartialEq, Clone)]
enum Several {
    Data(Payload),
    Empty1,
    Empty2,
}

#[derive(Debug, PartialEq, Clone)]
struct Payload {
    pointer: u64,
    flag: bool,
    text: String,
}

#[derive(Debug, PartialEq, Clone)]
enum Tagged {
    A(u8, u32),
    B(u16, [u8; 3]),
    C,
}

#[derive(Debug, PartialEq)]
enum Fieldless {
    X,
    Y,
    Z,
}

struct Mixed {
    a: u8,
    b: u64,
    c: u8,
}

struct Empty;

fn opt_ref(x: &u32, some: bool) -> Option<&u32> {
    if some {
        Some(x)
    } else {
        None
    }
}

fn main() {
    // Niche-filled: Option<bool>, Option<&T>, nested options.
    let mut flags: Vec<Option<bool>> = Vec::new();
    flags.push(Some(true));
    flags.push(None);
    flags.push(Some(false));
    println!("{} {} {}", flags.len(), flags[0].unwrap(), flags[2].unwrap());
    println!("{}", flags[1].is_none());

    let v = 41u32;
    println!("{} {}", opt_ref(&v, true).unwrap(), opt_ref(&v, false).is_none());

    let nested: Option<Option<u32>> = Some(Some(7));
    let outer_none: Option<Option<u32>> = None;
    let inner_none: Option<Option<u32>> = Some(None);
    println!("{} {} {}", nested.unwrap().unwrap(), outer_none.is_none(), inner_none.unwrap().is_none());

    // Data(Payload) plus two empty variants: the tag lives in a niche
    // inside Payload. Each variant must stay distinct after moves,
    // clones, and Vec storage, and the String must drop once.
    let mut many: Vec<Several> = Vec::new();
    many.push(Several::Empty1);
    many.push(Several::Data(Payload { pointer: 9, flag: true, text: String::from("hi") }));
    many.push(Several::Empty2);
    many.push(Several::Data(Payload { pointer: 3, flag: false, text: String::from("yo") }));
    let copy = many[1].clone();
    println!("{}", copy == many[1]);
    println!("{}", many[0] == many[2]);
    for s in &many {
        match s {
            Several::Data(p) => println!("data {} {} {}", p.pointer, p.flag, p.text),
            Several::Empty1 => println!("e1"),
            Several::Empty2 => println!("e2"),
        }
    }
    println!("{:?}", many[3]);

    // Tagged enums with reordered variant fields.
    let t1 = Tagged::A(200, 70000);
    let t2 = Tagged::B(500, [1, 2, 3]);
    println!("{:?} {:?} {}", t1, t2, t1 == Tagged::A(200, 70000));
    match t2 {
        Tagged::B(n, arr) => println!("b {} {}", n, arr[0] + arr[1] + arr[2]),
        _ => println!("other"),
    }
    println!("{:?}", Tagged::C);

    // Fieldless enum: tag only.
    println!("{} {:?}", Fieldless::Y == Fieldless::Y, Fieldless::Z);

    // Struct field reorder is internal; values read back as written.
    let m = Mixed { a: 1, b: 12345678901234, c: 2 };
    println!("{} {} {}", m.a, m.b, m.c);

    // A Vec of a zero-sized struct never allocates but tracks length.
    let mut empties: Vec<Empty> = Vec::new();
    empties.push(Empty);
    empties.push(Empty);
    empties.push(Empty);
    println!("{}", empties.len());
}

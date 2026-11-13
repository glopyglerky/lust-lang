// repr and explicit discriminants under layout-driven codegen: the
// tag stores the discriminant value at the repr width, matches read
// it back as the variant, and casts give the value.
#[repr(u16)]
#[derive(Debug, PartialEq)]
enum Code {
    Ok = 200,
    Moved(u8),
    Gone = 410,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum SignedTag {
    Neg = -7,
    Zero,
    Far = 100000,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
enum CEnum {
    A(u8),
    B,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum Memo {
    Unknown = 0,
    No = 1,
    Yes = 2,
}

#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,
    c: u16,
}

#[repr(C, align(32))]
struct Hot {
    tag: u8,
    memo: Memo,
}

fn main() {
    println!("{} {}", Code::Ok == Code::Ok, Code::Gone == Code::Moved(0));
    match Code::Moved(9) {
        Code::Moved(n) => println!("moved {}", n),
        other => println!("{:?}", other),
    }
    println!("{} {} {}", SignedTag::Neg as i32, SignedTag::Zero as i32, SignedTag::Far as i64);
    println!("{}", SignedTag::Neg as u64);
    let copied = SignedTag::Far;
    println!("{} {:?}", copied as i32, copied);

    println!("{:?} {}", CEnum::A(5), CEnum::B == CEnum::B);

    // Option over a repr(u64) enum niche-fills through the tag.
    let some: Option<Memo> = Some(Memo::Yes);
    let none: Option<Memo> = None;
    println!("{} {} {}", some.unwrap() as u64, none.is_none(), Memo::No as usize);

    // Packed fields read and write by value at unaligned offsets.
    let mut pk = Packed { a: 1, b: 70000, c: 513 };
    pk.b = pk.b + 1;
    let (a, b, c) = (pk.a, pk.b, pk.c);
    println!("{} {} {}", a, b, c);
    let mut pks: Vec<Packed> = Vec::new();
    pks.push(Packed { a: 2, b: 3, c: 4 });
    pks.push(Packed { a: 5, b: 6, c: 7 });
    let (a2, b2, c2) = (pks[1].a, pks[1].b, pks[1].c);
    println!("{} {} {}", a2, b2, c2);

    let hot = Hot { tag: 3, memo: Memo::Unknown };
    println!("{} {}", hot.tag, hot.memo as u64);
}

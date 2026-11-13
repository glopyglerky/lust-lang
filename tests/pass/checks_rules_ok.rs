#[repr(C, u8)]
enum E {
    A(u8) = 3,
    B = 5,
}

enum F {
    X = -1,
    Y,
}

fn take_slice(s: &[u8]) -> usize {
    s.len()
}

fn main() {
    println!("{} {} {}", true as u8, 'a' as u32, 65u8 as char);
    println!("{} {} {}", 3i32 as f64, 3.5f64 as i32, 2.5f64 as f32);
    println!("{} {}", 'a' as char, true as bool);
    let a = 3i32;
    println!("{} {} {}", (a as u8) << 2, a as u8 + 2, a as u8 > 2);
    let t = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9);
    println!("{} {}", t.0, t.10);
    let x = 'a: {
        break 'a 7;
    };
    println!("{}", x);
    'b: {
        loop {
            break;
        }
    }
    match 5 {
        5..=5 => println!("single"),
        _ => println!("other"),
    }
    match 4 {
        3..5 => println!("in"),
        _ => println!("out"),
    }
    let empty: [u8; 0] = [];
    let a2 = [];
    println!("{} {}", empty.len(), take_slice(&a2));
    let v: Vec<u8> = Vec::new();
    let mut v2 = Vec::new();
    v2.push(1u8);
    println!("{} {}", v.len(), v2.len());
    let o = Some(1);
    match o {
        Some(ref x) => println!("{}", x),
        None => {}
    }
    let o2 = &Some(2);
    match o2 {
        &Some(x) => println!("{}", x),
        _ => {}
    }
    let o3 = &Some(3);
    match o3 {
        Some(x) => println!("{}", x),
        None => {}
    }
    let e = E::A(9);
    match e {
        E::A(n) => println!("{}", n),
        E::B => println!("b"),
    }
    println!("{} {}", F::X as i32, F::Y as i32);
}

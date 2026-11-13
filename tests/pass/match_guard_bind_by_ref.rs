// Match guards read the arm's bindings by reference; the binding
// moves only in the arm body. A failed guard leaves the scrutinee
// whole, so a later arm can bind (and move) the same part, and the
// part drops once, with the arm that took it.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

impl D {
    fn get(&self) -> u8 {
        self.0
    }
    fn bump(&mut self) {
        self.0 += 1;
    }
}

struct S {
    a: u8,
    b: D,
}

struct Q {
    o: Option<D>,
    n: D,
}

// The review's D-3 shape: the guarded arm binds the part a later
// arm binds too, at a value below and above the guard's test.
fn later_arm_binds_same_part() {
    let o = Some(D(5));
    match o {
        Some(x) if x.0 > 10 => println!("big"),
        Some(x) => println!("small {}", x.0),
        None => {}
    }
    let o = Some(D(15));
    match o {
        Some(x) if x.get() > 10 => println!("big {}", x.0),
        Some(x) => println!("small {}", x.0),
        None => {}
    }
    println!("later end");
}

// Two guarded arms in a row, a nested pattern, and a tuple whose
// bindings drop in reverse order on the arm that takes them.
fn chains_and_nesting() {
    let o = Some(Some(D(7)));
    match o {
        Some(Some(x)) if x.0 > 10 => println!("a"),
        Some(Some(x)) if x.0 > 5 => println!("b {}", x.0),
        Some(Some(x)) => println!("c {}", x.0),
        _ => {}
    }
    let t = (D(1), D(2));
    match t {
        (x, y) if x.0 + y.0 > 10 => println!("a"),
        (x, y) => println!("b {} {}", x.0, y.0),
    }
    println!("chains end");
}

// A Copy field and a Drop field in one pattern; a guard temporary
// drops at the end of the guard, on both edges.
fn copy_and_drop_parts() {
    let s = S { a: 3, b: D(7) };
    match s {
        S { a, b } if a > 10 => println!("big {}", b.0),
        S { a, b } => println!("small {} {}", a, b.0),
    }
    let o = Some(D(5));
    match o {
        Some(x) if {
            let t = D(99);
            println!("guard {}", t.0);
            x.0 > 10
        } =>
        {
            println!("big")
        }
        Some(x) => println!("small {}", x.0),
        None => {}
    }
    println!("parts end");
}

// A `ref` binding in the guarded arm, a move in the next arm; a
// guarded match on a field place, with the rest of the struct
// alive after it; a match as a value.
fn ref_and_field() {
    let o = Some(D(5));
    match o {
        Some(ref x) if x.0 > 10 => println!("big {}", x.0),
        Some(x) => println!("small {}", x.0),
        None => {}
    }
    let q = Q { o: Some(D(2)), n: D(3) };
    match q.o {
        Some(x) if x.0 > 10 => println!("big"),
        Some(x) => println!("small {}", x.0),
        None => {}
    }
    println!("n {}", q.n.0);
    let o = Some(D(20));
    let r = match o {
        Some(x) if x.0 > 10 => x.0,
        Some(x) => x.0,
        None => 0,
    };
    println!("r {}", r);
}

// A guard in a loop (the guard result local is written every
// pass), a mutation through a `&mut` scrutinee in the arm body
// after a reading guard, and a fallback arm that moves nothing.
fn loops_and_mut() {
    let mut hit = 0u8;
    for i in 0..3u8 {
        let o = Some(D(i));
        match o {
            Some(x) if x.0 > 10 => hit = 9,
            Some(x) => hit = x.0,
            None => {}
        }
    }
    println!("hit {}", hit);
    let mut o = Some(D(5));
    match &mut o {
        Some(x) if x.0 < 10 => {
            x.bump();
            println!("in {}", x.0);
        }
        _ => {}
    }
    println!("after loop");
    let o = Some(D(5));
    match o {
        Some(x) if x.0 > 10 => println!("big"),
        _ => println!("fallback"),
    }
    println!("mut end");
}

// Owned fields: a String binding read in the guard and moved by
// the next arm.
fn owned() {
    let o = Some(String::from("hello"));
    match o {
        Some(x) if x.len() > 10 => println!("big"),
        Some(x) => println!("small {}", x),
        None => {}
    }
    let a = [String::from("x"), String::from("yy")];
    match a {
        [x, y] if x.len() > 10 => println!("big {}", y),
        [x, y] => println!("small {} {}", x, y),
    }
}

fn main() {
    later_arm_binds_same_part();
    chains_and_nesting();
    copy_and_drop_parts();
    ref_and_field();
    loops_and_mut();
    owned();
}

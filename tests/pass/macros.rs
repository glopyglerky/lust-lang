// macro_rules!: matching, transcription, repetitions, hygiene,
// fragment forwarding, and item generation in the supported subset.
// The output prints every observable result and must match rustc.

macro_rules! add {
    ($a:expr, $b:expr) => {
        $a + $b
    };
}

macro_rules! sum {
    ($($x:expr),*) => {
        0 $(+ $x)*
    };
    ($($x:expr);*) => {
        100 $(+ $x)*
    };
}

// A macro body's locals never capture caller locals of the same name,
// and an ident passed in binds the caller's variable.
macro_rules! with_tmp {
    ($e:expr) => {{
        let tmp = 100;
        tmp + $e
    }};
}

macro_rules! set_var {
    ($name:ident, $v:expr) => {
        $name = $v
    };
}

macro_rules! shadow_inside {
    () => {{
        let value = 1;
        value
    }};
}

// Forwarded fragments stay opaque: 3 as an expr must not re-match the
// inner macro's literal arm.
macro_rules! a_fwd {
    ($x:expr) => {
        b_fwd!($x)
    };
}
macro_rules! b_fwd {
    (3) => {
        3
    };
    ($x:expr) => {
        4
    };
}

// A forwarded ty is one unit to the inner matcher too.
macro_rules! ty_inner {
    (Vec<u8>) => {
        "split"
    };
    ($t:ty) => {
        "whole"
    };
}
macro_rules! ty_outer {
    ($t:ty) => {
        ty_inner!($t)
    };
}

// Item generation with a vis fragment and a meta repetition.
macro_rules! make {
    ($(#[$meta:meta])* $v:vis fn $name:ident() -> $t:ty { $e:expr }) => {
        $(#[$meta])*
        $v fn $name() -> $t { $e }
    };
}

make!(pub fn made() -> i64 { 5 });

pub struct Pair {
    pub lo: i64,
    pub hi: i64,
}

// A macro that writes an inherent impl.
macro_rules! pair_total {
    ($name:ident) => {
        impl $name {
            pub fn total(&self) -> i64 {
                self.lo + self.hi
            }
        }
    };
}

pair_total!(Pair);

// pat and block fragments through a match.
macro_rules! match_like {
    ($e:expr, $p:pat => $b:block, _ => $d:expr) => {
        match $e {
            $p => $b,
            _ => $d,
        }
    };
}

// A stmt fragment binds the name the caller passes in.
macro_rules! with_stmt {
    ($name:ident, $s:stmt) => {{
        $s
        $name
    }};
}

macro_rules! call_path {
    ($p:path, $v:expr) => {
        $p($v)
    };
}

// Nested repetitions and literal fragments.
macro_rules! count_tt {
    () => {
        0
    };
    ($head:tt $($tail:tt)*) => {
        1 + count_tt!($($tail)*)
    };
}

fn twice(x: i64) -> i64 {
    x * 2
}

fn main() {
    println!("add={} sum={} sum_semi={}", add!(20, 22), sum!(1, 2, 3), sum!(1; 2));

    let tmp = 5;
    println!("tmp kept={} with_tmp={}", tmp, with_tmp!(tmp));

    let mut target = 0;
    set_var!(target, 42);
    let value = 9;
    println!("assigned={} shadow={} caller={}", target, shadow_inside!(), value);

    println!("direct={} forwarded={}", b_fwd!(3), a_fwd!(3));
    println!("ty direct={} forwarded={}", ty_inner!(Vec<u8>), ty_outer!(Vec<u8>));

    println!("made={} path={}", made(), call_path!(twice, 21));

    let p = Pair { lo: 10, hi: 32 };
    println!("total={}", p.total());

    let some = Some(7);
    let v = match_like!(some, Some(n) => { n * 10 }, _ => 0);
    let none: Option<i64> = None;
    let w = match_like!(none, Some(n) => { n * 10 }, _ => 1);
    println!("match={} {}", v, w);

    let s = with_stmt!(doubled, let doubled = 21 * 2);
    println!("stmt={} count={}", s, count_tt!(a b c d));

    // A macro defined inside a function body is usable after it.
    macro_rules! local_mul {
        ($x:expr) => {
            $x * 3
        };
    }
    println!("local={}", local_mul!(14));

    // A body-local macro reads locals bound before its definition.
    let mut acc: i64 = 1;
    macro_rules! bump {
        ($v:expr) => {
            acc += $v
        };
    }
    bump!(10);
    bump!(31);
    println!("acc={}", acc);
}

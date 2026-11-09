// semantics: fragment specifiers ts-rust does not use but ARCH 6.2
// lists: pat, stmt, block, item, path, lifetime, vis. Plus nested
// repetition and the ? * + repetition operators.

macro_rules! match_like {
    ($e:expr, $p:pat => $b:block, _ => $d:expr) => {
        match $e {
            $p => $b,
            _ => $d,
        }
    };
}

// The ident must come from the caller: a name written here could
// not see the stmt fragment's binding (hygiene).
macro_rules! with_stmt {
    ($name:ident, $s:stmt) => {{
        $s
        $name
    }};
}

macro_rules! make_items {
    ($($i:item)*) => {
        $($i)*
    };
}

macro_rules! call_path {
    ($p:path, $v:expr) => {
        $p($v)
    };
}

macro_rules! first_of {
    ($lt:lifetime, $x:expr) => {{
        let out = $x;
        $lt: loop {
            break $lt;
        }
        out
    }};
}

macro_rules! vis_fn {
    ($v:vis fn $name:ident() -> $t:ty { $e:expr }) => {
        $v fn $name() -> $t { $e }
    };
}

// Nested repetition: rows of comma lists, the flags_macros shape.
macro_rules! table {
    ($($($c:literal),+);+) => {
        vec![$(vec![$($c),+]),+]
    };
}

macro_rules! at_least_one {
    ($($x:expr),+) => {
        0 $(+ $x)+
    };
}

make_items! {
    pub fn made_fn() -> i32 { 3 }
    pub const MADE_CONST: i32 = 4;
}
vis_fn!(pub fn vis_made() -> i32 { 5 });

fn twice(x: i32) -> i32 {
    x * 2
}

fn main() {
    let v = match_like!(Some(7), Some(n) => { n * 10 }, _ => 0);
    println!("pat/block={v}");
    let s = with_stmt!(doubled, let doubled = 21 * 2);
    println!("stmt={s}");
    println!("item={} {} vis={}", made_fn(), MADE_CONST, vis_made());
    println!("path={}", call_path!(twice, 21));
    println!("lifetime label={}", first_of!('a, 9));
    println!("nested={:?}", table!(1, 2; 3));
    println!("plus rep={}", at_least_one!(1, 2, 3));
}

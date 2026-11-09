// Raw identifiers can spell keywords, including ones reserved in edition 2024.
struct S {
    r#type: i64,
    r#gen: i64,
}

fn main() {
    let r#gen = 1;
    let r#match = 2;
    let s = S { r#type: 3, r#gen: 4 };
    println!("{} {} {} {}", r#gen, r#match, s.r#type, s.r#gen);
    println!("{}", r#fn());
}

fn r#fn() -> i64 {
    5
}

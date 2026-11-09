// The capacity word of a Vec/String is niche space above
// `isize::MAX`, so nested options of them need no extra tag, and the
// values still round-trip through those options.
enum E {
    U,
    V(i32),
    W { x: Vec<u8> },
}

fn main() {
    println!(
        "{} {} {} {}",
        std::mem::size_of::<Option<Option<Vec<u8>>>>(),
        std::mem::size_of::<Option<Option<String>>>(),
        std::mem::size_of::<Option<Result<Vec<u8>, u8>>>(),
        std::mem::size_of::<E>()
    );
    let a: Option<Option<Vec<u8>>> = Some(Some(vec![1, 2, 3]));
    let b: Option<Option<Vec<u8>>> = Some(None);
    let c: Option<Option<Vec<u8>>> = None;
    let d: Option<Option<String>> = Some(Some(String::from("hi")));
    let e = E::W { x: vec![9, 8] };
    let f = E::V(4);
    let g = E::U;
    let total = a.as_ref().map(|o| o.as_ref().map(|v| v.len()).unwrap_or(10)).unwrap_or(100)
        + b.as_ref().map(|o| o.as_ref().map(|v| v.len()).unwrap_or(10)).unwrap_or(100)
        + c.as_ref().map(|o| o.as_ref().map(|v| v.len()).unwrap_or(10)).unwrap_or(100)
        + d.as_ref().map(|o| o.as_ref().map(|s| s.len()).unwrap_or(10)).unwrap_or(100);
    let tag = match e {
        E::W { x } => x[0] as usize + x.len(),
        _ => 0,
    } + match f {
        E::V(n) => n as usize,
        _ => 0,
    } + match g {
        E::U => 1,
        _ => 0,
    };
    println!("{total} {tag} {} {:?}", d.as_ref().unwrap().as_deref().unwrap_or("?"), a.unwrap().unwrap());
}

enum E { A(String), B }

fn main() {
    // Guard over the middle of a slice pattern, shared slice.
    let v = [1, 2, 3, 4];
    match &v[..] {
        [first, rest @ .., last] if rest.len() > 1 => println!("a {} {} {:?}", first, last, rest),
        _ => println!("a no"),
    }
    // Owned array of Copy elements.
    match v {
        [first, rest @ .., last] if rest.len() > 1 => println!("b {} {} {:?}", first, last, rest),
        _ => println!("b no"),
    }
    // Guard failure leaves the scrutinee whole for the next arm.
    let w = [10, 20];
    match &w[..] {
        [first, rest @ .., last] if rest.len() > 5 => println!("c {} {}", first, last),
        [a, b] => println!("c {} {}", a, b),
        _ => println!("c no"),
    }
    // The guard indexes into the middle binding.
    let m = [1, 2, 3, 4];
    match m {
        [_, rest @ .., _] if rest[0] + rest[1] == 5 => println!("d {}", rest.len()),
        _ => println!("d no"),
    }
    // String elements behind a shared slice, guard reads element text.
    let ws = vec!["x".to_string(), "yy".to_string(), "z".to_string()];
    match &ws[..] {
        [first, rest @ .., last] if rest[0].len() == 2 => println!("e {} {}", first, last),
        _ => println!("e no"),
    }
    // A moved binding elsewhere in the pattern stays put on a
    // failed guard over the slice middle.
    let arr2 = [1, 2, 3];
    let t = (E::A("hi".to_string()), &arr2[..]);
    match t {
        (E::A(s), [_, rest @ .., _]) if rest.len() > 5 => println!("f1 {}", s),
        (E::A(s), _) => println!("f2 {}", s),
        _ => println!("f other"),
    }
}

// Std source methods of Option and Result in path form
// (fx-option-rest): `Option::map(x, f)` calls, methods as fn
// values, written type arguments, and the builtin iter forms.
fn main() {
    let o = Some(3);
    println!("{:?}", Option::map(o, |x| x + 1));
    println!("{:?}", Option::<i32>::map(o, |x| x * 2));
    println!("{:?}", <Option<i32>>::unwrap_or(o, 9));
    println!("{}", Option::is_some(&o));

    let v: Vec<Option<i32>> = vec![Some(1), None, Some(2)];
    println!("{:?}", v.iter().copied().filter(Option::is_some).collect::<Vec<_>>());
    println!("{}", v.iter().any(Option::is_none));
    let w: Vec<Option<i32>> = vec![Some(5), None];
    println!("{:?}", w.into_iter().map(Option::unwrap_or_default).collect::<Vec<_>>());

    let r: Vec<Result<i32, String>> = vec![Ok(1), Err(String::from("e"))];
    println!("{:?}", r.into_iter().map(Result::ok).collect::<Vec<_>>());
    let inner = Some(7);
    let nested: Option<&Option<i32>> = Some(&inner);
    println!("{}", nested.is_some_and(Option::is_some));
    println!("{}", None::<&Option<i32>>.is_none_or(Option::is_none));

    // The builtin iterator sources in path form.
    let q: Option<i32> = Some(11);
    println!("{:?}", Option::iter(&q).collect::<Vec<_>>());
    println!("{:?}", Option::into_iter(q).collect::<Vec<_>>());
    let mut mr: Result<i32, String> = Ok(12);
    println!("{:?}", Result::iter_mut(&mut mr).collect::<Vec<_>>());

    // A method fn value in a variable.
    let f = Option::<i32>::is_some;
    println!("{}", f(&Some(13)));
}

// Try `collect` into the other collection targets (F3): source
// collections (BTreeMap, BTreeSet) through their written-out
// impls in alloc, builtin inners (String, HashMap, VecDeque,
// Box<[T]>) through the expansion, and the `from_iter` spellings.
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

fn main() {
    let m: Result<BTreeMap<i32, &str>, &str> = vec![Ok((2, "b")), Ok((1, "a")), Ok((2, "B"))].into_iter().collect();
    println!("{:?}", m);
    let m_err: Result<BTreeMap<i32, i32>, &str> = vec![Ok((1, 10)), Err("no"), Ok((2, 20))].into_iter().collect();
    println!("{:?}", m_err);
    let s: Result<BTreeSet<i32>, ()> = vec![Ok(3), Ok(1), Ok(3)].into_iter().collect();
    println!("{:?}", s);
    let so: Option<BTreeSet<char>> = "abca".chars().map(Some).collect();
    println!("{:?}", so);
    let so_none: Option<BTreeSet<i32>> = vec![Some(1), None].into_iter().collect();
    println!("{:?}", so_none);
    let h: Result<HashMap<i32, i32>, &str> = vec![Ok((1, 2)), Ok((3, 4))].into_iter().collect();
    println!("hash {}", h.map(|m| m.len()).unwrap_or(0));
    let h_err: Result<HashMap<i32, i32>, &str> = vec![Ok((1, 2)), Err("x")].into_iter().collect();
    println!("{:?}", h_err.map(|m| m.len()));
    let text: Result<String, &str> = vec![Ok('a'), Ok('b')].into_iter().collect();
    println!("{:?}", text);
    let text_err: Option<String> = vec![Some('a'), None].into_iter().collect();
    println!("{:?}", text_err);
    let dq: Result<VecDeque<i32>, ()> = (0..4).map(Ok).collect();
    println!("{:?}", dq);
    let bx: Result<Box<[i32]>, ()> = vec![Ok(1), Ok(2)].into_iter().collect();
    println!("{:?}", bx);
    // The spelled-out from_iter forms build the same values.
    let f1 = Result::<Vec<i32>, &str>::from_iter(vec![Ok(1), Err("e")]);
    println!("{:?}", f1);
    let f2 = Option::<BTreeMap<i32, i32>>::from_iter(vec![Some((1, 2)), Some((0, 3))]);
    println!("{:?}", f2);
    let f3 = Result::<BTreeSet<i32>, &str>::from_iter([Ok(2), Ok(1)]);
    println!("{:?}", f3);
    // Nested: the inner failure is the outer Ok's value.
    let nested: Result<Result<Vec<i32>, &str>, &str> = vec![Ok(Ok(1)), Ok(Err("inner"))].into_iter().collect();
    println!("{:?}", nested);
    let nested2: Option<Option<Vec<i32>>> = vec![Some(Some(1)), Some(None)].into_iter().collect();
    println!("{:?}", nested2);
}

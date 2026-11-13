// Try `collect` in generic code (F3): over type parameters for
// the item and error, through adapter chains, returned with `?`,
// and inferred from the binding and the turbofish holes.
fn gather<T, E>(v: Vec<Result<T, E>>) -> Result<Vec<T>, E> {
    v.into_iter().collect()
}

fn maybe<T>(v: Vec<Option<T>>) -> Option<Vec<T>> {
    v.into_iter().collect()
}

fn run() -> Result<Vec<String>, String> {
    let words = ["a", "bb", "ccc"];
    let lens: Vec<usize> = words.iter().map(|w| Ok::<usize, String>(w.len())).collect::<Result<_, _>>()?;
    Ok(lens.iter().map(|n| n.to_string()).collect())
}

fn main() {
    println!("{:?}", gather::<i32, &str>(vec![Ok(1), Ok(2)]));
    println!("{:?}", gather::<i32, &str>(vec![Ok(1), Err("e")]));
    println!("{:?}", maybe(vec![Some("x"), Some("y")]));
    println!("{:?}", maybe::<i32>(vec![Some(1), None]));
    println!("{:?}", run());
    let inferred = (0..4).map(|i| if i < 3 { Ok(i * 2) } else { Err(i) }).collect::<Result<Vec<_>, _>>();
    println!("{:?}", inferred);
    let nums = [1, 2, 3, 4];
    let chained: Result<Vec<i32>, &str> = nums.iter().copied().filter(|x| x % 2 == 0).map(Ok).collect();
    println!("{:?}", chained);
    let rev: Option<Vec<i32>> = vec![Some(1), Some(2), Some(3)].into_iter().rev().collect();
    println!("{:?}", rev);
    let empty: Result<Vec<i32>, &str> = Vec::new().into_iter().collect();
    println!("{:?}", empty);
}

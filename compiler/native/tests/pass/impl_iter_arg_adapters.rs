// Iterator adapters and consumers on `impl Iterator` arguments: the
// template checks the chain over the bound, each instance over the
// concrete builtin iterator.
fn total(it: impl Iterator<Item = i32>) -> i32 { it.sum() }
fn evens(it: impl Iterator<Item = u32>) -> Vec<u32> { it.filter(|x| x % 2 == 0).collect() }
fn names<'a>(it: impl Iterator<Item = &'a str>) -> String { it.map(|s| s.to_uppercase()).collect::<String>() }
fn count_big(it: impl IntoIterator<Item = u8>) -> usize { it.into_iter().filter(|&b| b > 3).count() }
fn firsts<T: Clone>(it: impl Iterator<Item = (T, T)>) -> Vec<T> { it.map(|(a, _)| a).collect() }
fn maxlen(it: impl Iterator<Item = String>) -> Option<usize> { it.map(|s| s.len()).max() }
fn pos(mut it: impl Iterator<Item = i64>) -> Option<usize> { it.position(|x| x < 0) }
fn fold_it(it: impl Iterator<Item = u64>) -> u64 { it.fold(1, |a, b| a * b) }
fn zipped(a: impl Iterator<Item = i32>, b: impl Iterator<Item = char>) -> Vec<(i32, char)> { a.zip(b).collect() }
fn chained(a: impl Iterator<Item = i32>, b: Vec<i32>) -> i32 { a.chain(b.into_iter()).sum() }
fn en(it: impl Iterator<Item = char>) { for (i, c) in it.enumerate().skip(1).take(2) { println!("{} {}", i, c); } }
fn anyall(mut it: impl Iterator<Item = i32>) -> (bool, bool) { let a = it.any(|x| x > 2); (a, it.all(|x| x > 0)) }
fn lst(it: impl Iterator<Item = i32>) -> Option<i32> { it.last() }
fn pk(it: impl Iterator<Item = i32>) -> Option<i32> { let mut p = it.peekable(); let a = match p.peek() { Some(x) => Some(*x), None => None }; p.next(); a }
fn fm(it: impl Iterator<Item = &'static str>) -> Vec<i32> { it.filter_map(|s| if s.len() == 1 && s != "x" { Some(s.len() as i32) } else { None }).collect() }
fn cp<'a>(it: impl Iterator<Item = &'a i32>) -> i32 { it.copied().sum() }
fn cl<'a>(it: impl Iterator<Item = &'a String>) -> Vec<String> { it.cloned().collect() }
fn fe(it: impl Iterator<Item = i32>) { it.for_each(|x| print!("{} ", x)); println!(); }
fn mbk(it: impl Iterator<Item = (u8, &'static str)>) -> Option<(u8, &'static str)> { it.max_by_key(|p| p.0) }
fn tw(it: impl Iterator<Item = i32>) -> Vec<i32> { it.take_while(|&x| x < 3).collect() }
fn sb(it: impl Iterator<Item = i32>) -> Vec<i32> { it.step_by(2).collect() }
fn fnd(mut it: impl Iterator<Item = i32>) -> Option<i32> { it.find(|x| *x > 1) }
fn nth(mut it: impl Iterator<Item = i32>) -> Option<i32> { it.nth(1) }
fn insp(it: impl Iterator<Item = i32>) -> i32 { it.inspect(|x| print!("[{}]", x)).sum() }
fn prod(it: impl Iterator<Item = u32>) -> u32 { it.product() }
fn main() {
    println!("{}", total(vec![1, 2, 3].into_iter()));
    println!("{:?}", evens(0..10));
    println!("{}", names(vec!["a", "bc"].into_iter()));
    println!("{}", count_big(vec![1u8, 5, 9]));
    println!("{:?}", firsts(vec![(1, 2), (3, 4)].into_iter()));
    println!("{:?}", maxlen(vec![String::from("ab"), String::from("xyz")].into_iter()));
    println!("{:?}", pos(vec![3i64, -1].into_iter()));
    println!("{}", fold_it(1..6u64));
    println!("{:?}", zipped(0..3, "abc".chars()));
    println!("{}", chained(0..3, vec![10, 20]));
    en("wxyz".chars());
    println!("{:?}", anyall(vec![1, 3, 5].into_iter()));
    println!("{:?}", lst(0..4));
    println!("{:?}", pk(5..7));
    println!("{:?}", fm(vec!["1", "x", "3"].into_iter()));
    let v = vec![4, 5]; println!("{}", cp(v.iter()));
    let s = vec![String::from("q")]; println!("{:?}", cl(s.iter()));
    fe(1..4);
    println!("{:?}", mbk(vec![(1u8, "a"), (9, "b"), (3, "c")].into_iter()));
    println!("{:?}", tw(0..10));
    println!("{:?}", sb(0..7));
    println!("{:?}", fnd(0..4));
    println!("{:?}", nth(7..9));
    println!(" {}", insp(1..3));
    println!("{}", prod(1..5u32));
}

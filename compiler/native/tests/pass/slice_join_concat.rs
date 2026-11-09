// `[S]::join` and `concat` (tf-slice): the string family and
// the Vec family, with core's exact capacities and clone order.
fn main() {
    let v = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let j = v.join(",");
    println!("{j} {}", j.capacity());
    println!("{}", v.concat());
    let s: &[&str] = &["x", "y"];
    println!("{} {}", s.join("-"), s.concat());
    let e: Vec<String> = vec![];
    println!("'{}' {}", e.join(","), e.join(",").capacity());
    let one = vec!["only".to_string()];
    println!("{} {}", one.join(","), one.join(",").capacity());
    let w = vec![vec![1, 2], vec![3]];
    let c = w.concat();
    println!("{:?} {}", c, c.capacity());
    let j2 = w.join(&9);
    println!("{:?} {}", j2, j2.capacity());
    let arr = [[1, 2], [3, 4]];
    println!("{:?}", arr.join(&0));
    println!("{:?}", arr.join(&[0, 0][..]));
    println!("{:?}", arr.concat());
    let pa = ["a".to_string()];
    let pb = ["b".to_string(), "c".to_string()];
    let parts: Vec<&[String]> = vec![&pa[..], &pb[..]];
    println!("{:?}", parts.concat());
    let boxed: Vec<Box<[u8]>> = vec![vec![1u8, 2].into_boxed_slice(), vec![3u8].into_boxed_slice()];
    println!("{:?}", boxed.concat());
    let words = vec![vec!["a".to_string()], vec!["b".to_string()]];
    println!("{:?}", words.join(&"x".to_string()));
    println!("{:?}", w.join(&[7, 8][..]));
    let sep = vec![0];
    println!("{:?}", w.join(&sep[..]));
    let g: Vec<Vec<i32>> = vec![];
    println!("{:?} {:?}", g.concat(), g.join(&5));
}

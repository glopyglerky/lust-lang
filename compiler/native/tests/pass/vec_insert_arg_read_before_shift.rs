fn main() {
    let mut v = vec![9, 3, 5, 5];
    v.insert(0, v[1]);
    println!("{:?}", v);
    v.insert(v.len(), v[0]);
    println!("{:?}", v);
    let mut s = vec!["a".to_string(), "b".to_string()];
    s.insert(0, s[1].clone());
    println!("{:?}", s);
}

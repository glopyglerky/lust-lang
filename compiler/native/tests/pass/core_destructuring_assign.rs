struct G {
    n: i32,
}
impl Drop for G {
    fn drop(&mut self) {
        println!("drop {}", self.n);
    }
}
fn main() {
    let (mut a, mut b) = (String::from("x"), String::from("y"));
    (a, b) = (b.clone(), a.clone());
    println!("{} {}", a, b);
    _ = 42;
    let mut t = ((1, 2), 3);
    (t.0, _) = ((9, 8), 7);
    println!("{:?}", t);
    let mut arr = [G { n: 1 }, G { n: 2 }];
    [arr[0], _] = [G { n: 3 }, G { n: 4 }];
    println!("{}", arr[0].n);
    let mut s = String::from("old");
    (_, s) = (0, String::from("new"));
    println!("{}", s);
}

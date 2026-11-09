// AsRef, AsMut, Borrow, and ToOwned on std types, in generic code and
// with an expected type, plus a user AsRef impl.
use std::borrow::Borrow;

fn byte_len<B: AsRef<[u8]> + ?Sized>(b: &B) -> usize {
    b.as_ref().len()
}

fn shout<S: AsRef<str>>(s: S) -> String {
    let mut out = String::from(s.as_ref());
    out.push('!');
    out
}

fn first_of<S: AsRef<str>>(list: &[S]) -> String {
    list[0].as_ref().to_string()
}

fn find<K: Borrow<Q>, Q: PartialEq + ?Sized>(keys: &[K], q: &Q) -> Option<usize> {
    keys.iter().position(|k| k.borrow() == q)
}

struct Bytes(Vec<u8>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

fn fill<T: AsMut<[u8]>>(mut t: T) -> T {
    let s = t.as_mut();
    s[0] = 9;
    t
}

fn main() {
    println!("{} {} {}", byte_len("héllo"), byte_len(&String::from("ab")), byte_len(&vec![1u8, 2, 3]));
    println!("{} {}", byte_len(&Bytes(vec![7; 4])), byte_len(&[1u8, 2][..]));
    println!("{} {}", shout("hi"), shout(String::from("yo")));
    println!("{}", first_of(&["a", "b"]));
    println!("{}", first_of(&[String::from("x")]));
    let keys = vec![String::from("k1"), String::from("k2")];
    println!("{:?} {:?}", find(&keys, "k2"), find(&keys, "zz"));
    println!("{:?}", find(&[1, 2, 3], &3));
    let s = String::from("text");
    let r: &str = s.as_ref();
    let b: &[u8] = s.as_ref();
    let v = vec![1, 2];
    let sl: &[i32] = v.as_ref();
    let bx = Box::new(5);
    let inner: &i32 = bx.as_ref();
    println!("{} {:?} {:?} {}", r, b, sl, inner);
    let bs: &str = s.borrow();
    let o: String = "own".to_owned();
    let vv: Vec<i32> = sl.to_owned();
    println!("{} {} {:?}", bs, o, vv);
    println!("{:?}", fill(vec![0u8, 1]));
    // One impl (Box<String>: AsRef<String>) wins, then &String coerces.
    let bxs = Box::new(String::from("boxed"));
    let bs2: &str = bxs.as_ref();
    println!("{}", bs2);
}

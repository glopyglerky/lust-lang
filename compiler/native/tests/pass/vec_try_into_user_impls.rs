#[derive(Debug)]
struct Foo(usize);
impl TryFrom<Vec<u8>> for Foo {
    type Error = String;
    fn try_from(v: Vec<u8>) -> Result<Foo, String> { if v.is_empty() { Err("empty".into()) } else { Ok(Foo(v.len())) } }
}
trait FromJson: Sized { fn from_json(v: &[&str]) -> Result<Self, String>; }
impl FromJson for String { fn from_json(v: &[&str]) -> Result<Self, String> { Ok(v.concat()) } }
impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(v: &[&str]) -> Result<Self, String> { v.iter().map(|s| T::from_json(&[s])).collect() }
}
impl FromJson for [String; 2] {
    fn from_json(value: &[&str]) -> Result<Self, String> {
        Vec::from_json(value)?
            .try_into()
            .map_err(|items: Vec<_>| format!(": expected 2 items, found {}", items.len()))
    }
}
fn conv(v: Vec<u8>) -> Result<Foo, String> { let f: Foo = v.try_into()?; Ok(f) }
fn main() {
    let f: Result<Foo, String> = vec![1u8, 2].try_into();
    println!("{:?}", f);
    let g = Foo::try_from(Vec::new());
    println!("{:?}", g);
    let h: Foo = vec![3u8].try_into().unwrap();
    println!("{:?} {:?}", h, conv(vec![]));
    println!("{:?}", <[String; 2]>::from_json(&["a", "b"]));
    println!("{:?}", <[String; 2]>::from_json(&["a"]));
}

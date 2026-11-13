struct P { text: Option<Box<str>> }
fn main() { let p = P { text: Some("ab".into()) }; println!("{:?}", p.text.as_deref()); let q = P { text: None }; println!("{:?}", q.text); }

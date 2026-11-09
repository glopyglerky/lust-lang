// `@` bindings rustc accepts: Copy values, all-`ref` parts, and
// default binding modes through a shared reference.
fn f1() { let x = (String::new(), 2); match x { ref a @ (_, n) => { println!("{}", n); }, } }
fn f2() { let x = (1, 2); match x { a @ (b, c) => { println!("{} {} {:?}", b, c, a); }, } }
fn f3() { let x = (1, 2); match x { a @ (ref b, _) => { println!("{} {:?}", b, a); }, } }
fn f4() { let x = Some(String::from("a")); match &x { a @ Some(b) => { println!("{:?} {}", a, b); }, _ => {} } }
fn f5() { let x = (1, 2); match x { ref a @ (ref b, ref c) => { println!("{:?} {} {}", a, b, c); }, } }
fn main() { f1(); f2(); f3(); f4(); f5(); }

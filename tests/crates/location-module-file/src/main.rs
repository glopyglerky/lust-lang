mod loc;

fn main() {
    let l = loc::here();
    println!("{}:{}:{}", l.file(), l.line(), l.column());
    println!("{}", loc::from_module());
    println!("{}", loc::through_closure());
    let w = loc::wrapper();
    println!("{} {}", w.file(), w.line());
    eprintln!("about to fail");
    loc::fail("the end");
}

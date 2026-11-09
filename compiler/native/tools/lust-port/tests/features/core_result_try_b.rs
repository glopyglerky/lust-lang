// feature: core.result_try
// form: Box value with deref and a recursive Box field
enum List { Cons(i32, Box<List>), Nil }
fn len(l: &List) -> i32 { match l { List::Cons(_, next) => 1 + len(next), List::Nil => 0 } }
fn main() {
    let b = Box::new(10);
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
    println!("{} {}", *b, len(&list));
}

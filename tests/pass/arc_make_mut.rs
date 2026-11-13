use std::sync::Arc;
fn main() {
    let mut a = Arc::new(String::from("one"));
    let w = Arc::downgrade(&a);
    Arc::make_mut(&mut a).push_str("+two");
    println!("{} {:?} {}", a, w.upgrade(), Arc::weak_count(&a));
    let b = a.clone();
    Arc::make_mut(&mut a).push('!');
    println!("{} {} {} {}", a, b, Arc::strong_count(&a), Arc::strong_count(&b));
    let mut c = Arc::new(1);
    *Arc::make_mut(&mut c) += 1;
    println!("{}", c);
}

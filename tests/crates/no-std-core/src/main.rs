use no_std_core::{Outcome, Pair, checked, first, shadowed_names, view, width};

fn main() {
    let values = [10, 20, 12];
    let window = view(&values);
    let pair = Pair { left: 7, right: 9 };
    assert_eq!(pair, pair.clone());
    assert_eq!(window.sum(), 42);
    assert_eq!(width::<3>(), values.len());
    match first(&values) {
        Outcome::Value(value) => assert_eq!(value, 10),
        Outcome::Empty => panic!("missing first value"),
    }
    assert_eq!(checked(42), Ok(42));
    assert_eq!(checked(100), Err(()));
    assert_eq!(shadowed_names(), 42);
    assert_eq!(no_std_core::local_macro(), 42);
    println!("{:?} {} {}", pair, window.sum(), width::<3>());
}

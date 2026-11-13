use crate_root_types::{Fault, resolution};

fn main() {
    assert_eq!(resolution::associated(), 0);
    assert_eq!(resolution::classify(Fault::Code(42)), 42);
    match resolution::result() {
        Err(value) => assert_eq!(value.code(), 42),
        Ok(()) => panic!("expected a code"),
    }
    let generic = resolution::generic_shadow::<()>();
    assert_eq!(generic.width(), 2);
    assert_eq!(generic.values[0] + generic.values[1], 42);
    let typed = resolution::typed();
    assert_eq!(typed.width(), 3);
    assert_eq!(typed.values[0] + typed.values[1] + typed.values[2], 42);
    let block = resolution::block_shadow();
    assert_eq!(block.values[0] + block.values[1], 42);
    println!("{} {} {}", generic.width(), typed.width(), block.width());
}

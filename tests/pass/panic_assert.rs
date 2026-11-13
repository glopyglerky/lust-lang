// assert_eq! failure aborts.
#[derive(Debug, PartialEq)]
enum Mode {
    On,
    Off,
}

fn main() {
    assert_eq!(1 + 1, 2);
    println!("ok so far");
    assert_eq!(Mode::On, Mode::Off, "modes differ");
}

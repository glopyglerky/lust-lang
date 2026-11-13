// expect: L0112
#[derive(PartialEq)]
struct Item(u8);
fn main() {
    let values = [Item(2), Item(5)];
    assert_eq!(&values[..], &[Item(2), Item(5)]);
}

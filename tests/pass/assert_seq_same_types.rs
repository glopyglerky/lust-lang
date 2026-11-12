// Same-type assertions keep their original comparison and Debug path.
// Cross-type sequence support must not change generic or string assertions.
fn compare<T: PartialEq + core::fmt::Debug>(left: T, right: T) {
    assert_eq!(left, right);
}

#[derive(PartialEq, Debug)]
struct Item(u8);

fn main() {
    compare(4u8, 4);
    compare(Item(4), Item(4));
    compare([2u8, 5], [2, 5]);
    compare(&[2u8, 5][..], &[2, 5][..]);
    compare(vec![2u8, 5], vec![2, 5]);
    compare(String::from("same"), String::from("same"));
    compare("same", "same");
    assert_ne!([2u8, 5], [2, 9]);
    assert_ne!(&[2u8, 5][..], &[2, 9][..]);
    assert_ne!(vec![2u8, 5], vec![2, 9]);
    assert_eq!(String::from("text"), "text");
    assert_eq!("text", String::from("text"));
    assert_ne!(String::from("text"), "other");
    println!("same-type assertions passed");
}

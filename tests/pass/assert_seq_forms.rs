// Assertions use the same asymmetric sequence equality impls as == and !=,
// borrow each operand once, and retain its original type for Debug.
fn compare<T: PartialEq + core::fmt::Debug, const N: usize>(left: &[T], right: &[T; N]) {
    assert_eq!(left, right);
}

fn slice_once<'a>(count: &mut u8, values: &'a [u8]) -> &'a [u8] {
    *count += 1;
    values
}

fn array_once(count: &mut u8) -> [u8; 2] {
    *count += 10;
    [2, 5]
}

#[derive(PartialEq, Debug)]
struct Item(u8);

fn main() {
    let values = [2u8, 5, 8];
    let slice = &values[..2];
    assert_eq!(slice, &[2, 5]);
    assert_eq!(&[2, 5], slice);
    assert_eq!(slice, [2, 5]);
    assert_eq!([2, 5], slice);
    assert_ne!(&values[..1], &[2, 5]);
    assert_ne!(slice, &[2, 9]);
    assert_eq!(&values[..0], &[]);
    assert_eq!(&slice, &&[2, 5]);
    let mut count = 0;
    assert_eq!(slice_once(&mut count, slice), array_once(&mut count));
    assert_eq!(count, 11);
    compare(slice, &[2, 5]);
    compare(&[Item(2), Item(5)], &[Item(2), Item(5)]);
    let vector = vec![2u8, 5];
    assert_eq!(vector, [2, 5]);
    assert_eq!(vector, &[2, 5]);
    assert_eq!(vector, slice);
    assert_eq!(slice, vector);
    assert_ne!(&[f64::NAN][..], &[f64::NAN]);
    println!("{} {}", count, vector.len());
}

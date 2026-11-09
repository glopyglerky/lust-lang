fn main() {
    let mut values = vec![1];
    let borrowed = &values;
    values.push(2);
    println!("{borrowed:?}");
}

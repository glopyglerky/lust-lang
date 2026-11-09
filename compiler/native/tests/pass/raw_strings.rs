fn main() {
    let a = r"plain \n text";
    let b = r#"with "quotes""#;
    let c = r##"with "# inside"##;
    println!("{}", a);
    println!("{}", b);
    println!("{}", c);
}

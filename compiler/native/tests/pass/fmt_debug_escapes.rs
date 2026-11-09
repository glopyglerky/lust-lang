fn main() {
    let s = String::from("tab\there \"q\" back\\ nl\n uni\u{7f} é \u{200b} \0");
    println!("{:?}", s);
    println!("{:?} {:?} {:?} {:?} {:?}", '\0', '\'', '"', '\u{7f}', 'é');
    println!("{:?} {:?} {:?} {:?} {:?}", '\u{300}', '\u{200d}', '\u{fe0f}', '\u{e0001}', '\u{a0}');
    println!("{:?} {:?} {:?}", '\u{2028}', '\u{feff}', ' ');
    let v = vec![String::from("a"), String::from("b\"")];
    println!("{:?}", v);
}

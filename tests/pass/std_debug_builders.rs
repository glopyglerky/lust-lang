// Std Debug impls that std derives or writes with the builders: `{:#?}`
// and widths reach the inner fields (fx-splice-derive).
fn main() {
    println!("{:?} {:#?}", 'A'.to_lowercase(), 'A'.to_lowercase());
    println!("{:?} {:#?}", 'ß'.to_uppercase(), 'x'.to_uppercase());
    let e = "x".parse::<std::net::IpAddr>().unwrap_err();
    println!("{:?} {:#?}", e, e);
    let e = "x".parse::<std::net::SocketAddrV4>().unwrap_err();
    println!("{:?}", e);
    let s = std::process::Command::new("false").status().unwrap();
    println!("{:?} {:#?} {:5?}", s, s, s);
    let mut lower = 'A'.to_lowercase();
    lower.next();
    println!("{:?}", lower);
}

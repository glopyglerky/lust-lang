// Turbofish on builtin constructor patterns (`Ok::<u8, String>(x)`,
// `None::<u8>`) matches like the expression forms.
fn main() {
    let r: Result<u8, String> = Ok(3);
    match r {
        Ok::<u8, String>(x) => println!("{}", x),
        Err::<_, String>(e) => println!("{}", e),
    }
    let o: Option<u8> = None;
    if let None::<u8> = o {
        println!("none");
    }
}

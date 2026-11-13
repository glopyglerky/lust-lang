// `Box<dyn Error>` conversions (F3): from an error value, from
// `&str`, `String`, and `Cow<str>` (Display and Debug are the
// string's own, as in std), and `?` into a boxed error return.
use std::borrow::Cow;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct Boom(u32);
impl fmt::Display for Boom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "boom {}", self.0) }
}
impl Error for Boom {}

fn may_io() -> Result<(), Box<dyn Error>> {
    let _ = std::fs::read("/nonexistent-lust-f3")?;
    Ok(())
}

fn may_str() -> Result<(), Box<dyn Error>> {
    Err("plain words")?
}

fn may_boom() -> Result<(), Box<dyn Error + Send + Sync>> {
    Err(Boom(9))?
}

fn generic<E: Error + 'static>(e: E) -> Box<dyn Error> {
    e.into()
}

fn main() {
    let a: Box<dyn Error> = "words".into();
    println!("{a} {a:?} {:?}", a.source());
    let b: Box<dyn Error> = String::from("owned words").into();
    println!("{b} {b:?}");
    let c: Box<dyn Error + Send + Sync> = Cow::Borrowed("cow words").into();
    println!("{c} {c:?}");
    let d: Box<dyn Error> = Boom(3).into();
    println!("{d} {d:?}");
    println!("{}", generic(Boom(4)));
    println!("io: {}", may_io().unwrap_err());
    println!("str: {}", may_str().unwrap_err());
    println!("boom: {}", may_boom().unwrap_err());
    let from_fn: Box<dyn Error> = Box::from(Boom(11));
    println!("from: {from_fn} {}", from_fn.to_string());
    let from_str: Box<dyn Error + Send + Sync> = Box::from("from words");
    println!("from str: {}", from_str.to_string());
    let as_ref: &dyn Error = &Boom(12);
    println!("ref: {}", as_ref.to_string());
    let boxed: Box<Boom> = Box::new(Boom(5));
    println!("box source: {:?}", boxed.source());
    let r = &Boom(6);
    println!("ref source: {:?}", r.source());
}

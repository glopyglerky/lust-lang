// An impl's type parameter shadows a user struct of the same name:
// `&mut R` in `impl<R: Rd> Rd for &mut R` is the parameter, not
// `struct R<'a>` (whose hidden lifetime would be an error).
struct R<'a>(&'a i32);

trait Rd {
    fn r(&self) -> u8;
}

impl<R: Rd + ?Sized> Rd for &mut R {
    fn r(&self) -> u8 {
        (**self).r()
    }
}

impl Rd for u8 {
    fn r(&self) -> u8 {
        *self
    }
}

fn main() {
    let mut x = 5u8;
    let y = &mut x;
    println!("{}", y.r());
    let z = 1;
    println!("{}", R(&z).0);
}

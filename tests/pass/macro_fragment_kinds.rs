// Fragment kinds must classify by rustc's full grammar, not by
// what lust's subset parser accepts: these fragments are only
// classified (the arm discards them), so the program runs the
// same under rustc and lust.
macro_rules! ke { ($e:expr) => { "expr" }; ($($t:tt)*) => { "tts" }; }
macro_rules! kp { ($p:pat) => { "pat" }; ($($t:tt)*) => { "tts" }; }
macro_rules! kt { ($t:ty) => { "ty" }; ($($t:tt)*) => { "tts" }; }
macro_rules! ks { ($s:stmt) => { "stmt" }; ($($t:tt)*) => { "tts" }; }
macro_rules! cnt { () => { 0 }; ($h:tt $($t:tt)*) => { 1 + cnt!($($t)*) }; }
macro_rules! shr { (>> $x:ident) => { "shr" }; (> > $x:ident) => { "gtgt" }; ($($t:tt)*) => { "tts" }; }
macro_rules! dsep { ($($x:ident)*) => { stringify!($($x)$*) }; }
macro_rules! unit { () => { () }; }

fn main() {
    println!("{}", ke!(|x: i32| x + 1));
    println!("{}", ke!(move || 1));
    println!("{}", ke!(async { 1 }));
    println!("{}", ke!(unsafe { 1 }));
    println!("{}", ke!(x?));
    println!("{}", ke!(x.await));
    println!("{}", ke!(&raw const x));
    println!("{}", ke!(#[allow(unused)] 1));
    println!("{}", ke!(const { 1 }));
    println!("{}", kp!(1.0));
    println!("{}", kp!(box x));
    println!("{}", kp!(..=5));
    println!("{}", kp!(x @ 1..=5));
    println!("{}", kp!(S { a, .. }));
    println!("{}", kt!(dyn));
    println!("{}", kt!(dyn Drop));
    println!("{}", kt!(impl Drop));
    println!("{}", kt!(fn(u8) -> u8));
    println!("{}", ks!(let Some(x) = y else { return }));
    println!("{}", ks!(struct S;));
    println!("{}", ks!(let x = 1));
    println!("{}", cnt!(x >> y));
    println!("{}", cnt!(x > > y));
    println!("{}", cnt!(x >>= y));
    println!("{}", shr!(>> a));
    println!("{}", shr!(> > a));
    println!("{}", dsep!(a b));
    unit!{} - 1;
    println!("ok");
}

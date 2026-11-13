macro_rules! s { ($e:expr) => { stringify!($e) }; }
macro_rules! m { ($($a:ident)=>*) => { stringify!($($a)-*) }; }
macro_rules! dc { () => { stringify!($crate::x) }; }
macro_rules! k { ($i:ident) => { stringify!($i) }; }

fn main() {
    println!("{}", stringify!(a+b));
    println!("{}", stringify!(a + b));
    println!("{}", stringify!(a  ,  b));
    println!("{}", stringify!(x.y()));
    println!("{}", stringify!(r"raw" b"bs" 1u8));
    println!("{}", stringify!(f(a,b)[0]));
    println!("{}", stringify!({ a; b }));
    println!("{}", stringify!(f( a )));
    println!("{}", stringify!(f (a)));
    println!("{}", stringify!([ 1 ,2 ]));
    println!("{}", stringify!(a ;b));
    println!("{}", stringify!(x . y));
    println!("{}", s!(a  .  b ( c )));
    println!("{}", s!(f( a , b )));
    println!("{}", m!(a => b => c));
    println!("{}", dc!());
    let r#type = 3;
    println!("{} {}", k!(r#type), r#type);
    println!("{}", stringify!(
        /// hi
        x));
}

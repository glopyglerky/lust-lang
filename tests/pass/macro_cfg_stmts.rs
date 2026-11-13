macro_rules! m { () => { #[cfg(all())] let y = 6; #[cfg(any())] let y = 5; println!("{}", y); }; }
#[cfg_attr(all(), macro_export)]
macro_rules! three { () => { 3 }; }

#[derive(Debug)]
enum E { A, #[cfg(any())] B }

#[cfg(any())]
undefined_macro!();

fn main() {
    let x = 1;
    #[cfg(any())]
    let x = 2;
    println!("{}", x);
    #[cfg(any())]
    println!("off");
    #[cfg(all())]
    println!("on");
    let v = 1;
    match v {
        #[cfg(any())]
        1 => println!("off arm"),
        _ => println!("other"),
    }
    let mut n = 0;
    #[cfg(any())]
    {
        n += 100;
    }
    n += 1;
    println!("{}", n);
    m!();
    #[cfg(any())]
    undefined_macro!();
    println!("ok");
    let e = E::A;
    match e {
        E::A => println!("{:?}", e),
    }
    let v2 = {
        #[cfg(any())]
        { 1 }
        #[cfg(all())]
        { 2 }
    };
    println!("{}", v2);
    println!("{}", crate::three!());
}

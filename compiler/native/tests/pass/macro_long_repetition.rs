// Fragment parses in a long repetition take time in proportion to its
// length. Each fragment's end search used to run over the whole rest of
// the input, so these 1024-item calls took minutes (`ty`, `pat`, `stmt`)
// or grew quadratically (`expr`, `item`).
macro_rules! tys { ($($l:lifetime : $t:ty),* $(,)?) => { [$(stringify!($t)),*].len() } }
macro_rules! pats { ($($p:pat),* $(,)?) => { [$(stringify!($p)),*].len() } }
macro_rules! stmts { ($($s:stmt),* $(,)?) => { [$(stringify!($s)),*].len() } }
macro_rules! exprs { ($($e:expr),* $(,)?) => { [$(stringify!($e)),*].len() } }
macro_rules! items { ($($i:item)*) => { [$(stringify!($i)),*].len() } }
// double!(mac [tokens] x x ..) calls mac! with the tokens 2^(count of x) times.
macro_rules! double {
    ($m:ident [$($t:tt)*] x $($rest:tt)*) => { double!($m [$($t)* $($t)*] $($rest)*) };
    ($m:ident [$($t:tt)*]) => { $m!($($t)*) };
}
fn main() {
    println!("{}", double!(tys ['a: Vec<u8>,] x x x x x x x x x x));
    println!("{}", double!(pats [Some(x),] x x x x x x x x x x));
    println!("{}", double!(stmts [let x = 1,] x x x x x x x x x x));
    println!("{}", double!(stmts [let f = |x: u8| x + 1,] x x x x x x x x x x));
    println!("{}", double!(exprs [|x: u8| x | 1,] x x x x x x x x x x));
    println!("{}", double!(exprs [Vec::<u8>::new().len() < 2,] x x x x x x x x x x));
    println!("{}", double!(items [struct S;] x x x x x x x x x x));
}

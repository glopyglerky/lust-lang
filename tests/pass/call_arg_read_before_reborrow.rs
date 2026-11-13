// A call that reads a place behind a `&mut` (`c.program`) before a later argument reborrows it
// (`c`): the read happens first, as in rustc (ts-rust effect/ls/document_symbols.rs:40).
use std::cell::RefCell;
struct GoProgram { n: u32 }
struct Checker { program: &'static GoProgram, k: u32 }
struct TypeParser<'c> { program: &'static GoProgram, checker: &'c mut Checker }
impl<'c> TypeParser<'c> {
    fn new(program: &'static GoProgram, checker: &'c mut Checker) -> Self { TypeParser { program, checker } }
    fn run(&mut self) -> u32 { self.checker.k += 1; self.program.n + self.checker.k }
}
static P: GoProgram = GoProgram { n: 5 };
fn get() -> (RefCell<Checker>, u8) { (RefCell::new(Checker { program: &P, k: 0 }), 0) }
fn main() {
    let (checker, _done) = get();
    let c = &mut *checker.borrow_mut();
    let mut tp = TypeParser::new(c.program, c);
    println!("{}", tp.run());
}

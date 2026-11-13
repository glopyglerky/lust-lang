// expect: L0110
// The operand of `&` is no coercion site: `&&[1]` is not a `&&[u16]`.
const RC: &&[u16] = &&[1];
fn main() {
    println!("{}", RC.len());
}

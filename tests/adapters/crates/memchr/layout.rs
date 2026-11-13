// lust-reject: L0652
// The adapter's Finder does not have upstream's layout, so lust must
// not answer size_of for it (ARCH 3.6); the real crate prints its size.
// The size is a const: an alias with a lifetime in a turbofish inside a
// body is L0203 before the layout check runs.
type F = memchr::memmem::Finder<'static>;
const SIZE: usize = std::mem::size_of::<F>();

fn main() {
    println!("{}", SIZE > 0);
}

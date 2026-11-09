// Panic columns count display width as rustc does: a tab is 4,
// a wide char 2, an accented letter 1.
// panic: attempt to add with overflow
fn main() {
    let n = std::env::args().count() as u8;
	let a = ("é→日🦀", n + 1);
	println!("{:?}", a);
	let b = ("日本", n + 255);
    println!("{:?}", b);
}

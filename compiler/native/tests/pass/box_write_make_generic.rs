// #1 census cause (187k lines): checker/types.rs ArenaBox::new_with builds an owned value with
// Box::write(Box::new_uninit(), make()).
fn make_box<T>(make: impl FnOnce() -> T) -> Box<T> {
    Box::write(Box::new_uninit(), make())
}

fn main() {
    let b = make_box(|| vec![1u32, 2, 3]);
    let s = make_box(|| String::from("arena"));
    println!("{:?} {} {}", b, s, b.len() + s.len());
}

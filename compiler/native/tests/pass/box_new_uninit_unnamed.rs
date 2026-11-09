// No `MaybeUninit` and no `.write` in the text: the uninit constructors
// alone must still make a `MaybeUninit<T>` slot of T's size, with Debug.
fn main() {
    let b = Box::<u64>::new_uninit();
    println!("{:?}", b);
    let z = Box::<[u64; 4]>::new_zeroed();
    println!("{:?}", z);
    let s = Box::write(Box::new_uninit(), String::from("slot"));
    let v = Box::write(Box::new_uninit(), vec![1u64, 2, 3, 4, 5]);
    println!("{} {:?}", s, v);
    let n = Box::<[String]>::new_uninit_slice(2);
    println!("{}", n.len());
}

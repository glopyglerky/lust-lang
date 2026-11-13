// MGP (ARCH 9.3): one drop of a generic value. The Drop impl
// prints, so the output pins that the drop ran, ran once, and
// ran at scope exit for each instance. Two different type
// arguments give two instances of the generic drop glue.
struct Guard<T> {
    value: T,
    label: &'static str,
}
impl<T: Copy> Guard<T> {
    fn new(value: T, label: &'static str) -> Self {
        Guard { value, label }
    }
    fn get(&self) -> T {
        self.value
    }
}
impl<T> Drop for Guard<T> {
    fn drop(&mut self) {
        println!("drop {}", self.label);
    }
}
fn main() {
    let a: Guard<u8> = Guard::new(3, "a");
    println!("value {}", a.get());
    {
        let b: Guard<u32> = Guard::new(4, "b");
        println!("value {}", b.get());
    }
    println!("after inner");
}

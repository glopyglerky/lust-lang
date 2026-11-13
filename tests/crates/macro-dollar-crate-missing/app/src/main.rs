#[macro_export]
macro_rules! only_app {
    () => (1u8);
}

fn main() {
    println!("{}", base::call_app!());
}

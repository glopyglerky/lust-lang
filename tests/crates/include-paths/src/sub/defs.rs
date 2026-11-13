// The include in this macro body resolves relative to the file of the
// macro call site (rustc), so a call in main.rs reads src/calldata.txt,
// not the calldata.txt next to this definition.
#[macro_export]
macro_rules! load_calldata {
    () => {
        include_str!("calldata.txt")
    };
}

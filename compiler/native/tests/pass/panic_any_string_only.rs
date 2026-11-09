// `panic_any` as the only statement: its synthetic `dyn Any + Send`
// vtable still gets built, and the String payload is the message.
fn main() {
    std::panic::panic_any(String::from("owned"));
}

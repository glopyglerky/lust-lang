use std::any::Any;
mod m { pub const R: u32 = 1; }
pub trait AnyValue: Any {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any> AnyValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
fn wrap<T: Copy>(t: T) -> T { t }
fn main() { let x = 5i32; println!("{}", x.as_any().is::<i32>() as u32 + m::R + wrap(1)); }

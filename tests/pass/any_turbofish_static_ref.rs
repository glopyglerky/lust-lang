// `'static` on a reference in the turbofish of a `core::any` method is the elided form,
// since `T: 'static` there (ts_goport gostd/context.rs:1040).
use std::any::Any;
use std::sync::Arc;
struct V {
    val: Arc<dyn Any + Send + Sync>,
}
impl V {
    fn show(&self) -> String {
        if let Some(s) = self.val.downcast_ref::<String>() {
            return s.clone();
        }
        if let Some(s) = self.val.downcast_ref::<&'static str>() {
            return (*s).to_string();
        }
        "other".to_string()
    }
}
fn main() {
    let b = V { val: Arc::new(String::from("yo")) };
    let c = V { val: Arc::new(3u8) };
    let d: Box<dyn Any> = Box::new(1u8);
    println!("{} {} {} {:?}", b.show(), c.show(), d.is::<&'static str>(), d.downcast_ref::<&'static [&'static str]>());
}

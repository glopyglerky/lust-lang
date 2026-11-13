// `impl Logger for Option<Rc<dyn Logger>>` is for a `'static` object, and `&self.logger` (a field)
// already has one, so the closure's own `Rc<dyn Logger>` param does not matter (ts-rust
// project/session.rs update_watches, trigger_ata_for_updated_projects).
use std::rc::Rc;
trait Logger { fn log(&self, msg: &str); }
struct L;
impl Logger for L { fn log(&self, msg: &str) { println!("log {msg}"); } }
impl Logger for Option<Rc<dyn Logger>> {
    fn log(&self, msg: &str) { if let Some(l) = self { l.log(msg); } }
}
fn log_to(l: &dyn Logger, m: &str) { l.log(m); }
struct Sess { logger: Option<Rc<dyn Logger>> }
impl Sess {
    fn run(&self, x: &u32) {
        let local: Rc<dyn Logger> = Rc::new(L);
        let f = |n: &u32, d: Rc<dyn Logger>| { d.log("in"); log_to(&self.logger, &format!("{n}")); };
        f(x, local);
        log_to(&self.logger, "out");
    }
}
fn main() { let s = Sess { logger: Some(Rc::new(L)) }; s.run(&4); }

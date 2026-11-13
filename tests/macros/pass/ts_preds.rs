// macro: preds
// source: crates/ts_goport/src/format/rules.rs:8
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::sync::Arc;
pub struct Ctx { pub value: i32 }
pub type ContextPredicate = Arc<dyn Fn(&mut Ctx) -> bool + Send + Sync>;

fn is_positive(ctx: &mut Ctx) -> bool { ctx.value > 0 }
fn is_even(ctx: &mut Ctx) -> bool { ctx.value % 2 == 0 }
fn at_least(min: i32) -> ContextPredicate { Arc::new(move |ctx: &mut Ctx| ctx.value >= min) }

macro_rules! preds {
    (@one $f:ident ( $($args:tt)* )) => { $f($($args)*) };
    (@one $f:ident) => { Arc::new($f) as ContextPredicate };
    ($($f:ident $(( $($args:tt)* ))?),* $(,)?) => {
        vec![$(preds!(@one $f $(( $($args)* ))?)),*]
    };
}

fn main() {
    let ps: Vec<ContextPredicate> = preds![is_positive, at_least(3), is_even];
    let mut ctx = Ctx { value: 4 };
    let r: Vec<bool> = ps.iter().map(|p| p(&mut ctx)).collect();
    println!("preds={r:?} count={}", ps.len());
    let one: Vec<ContextPredicate> = preds![is_positive,];
    println!("one={}", one.len());
}

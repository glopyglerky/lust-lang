// R24: a typed context key over an `Arc<dyn Any + Send + Sync>` value, as
// ts-rust's gostd::context reads it (`ctx.value(&KEY)`, then a deref).
use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct ContextKey<T> {
    name: &'static str,
    _value: PhantomData<fn() -> T>,
}

impl<T: 'static> ContextKey<T> {
    pub const fn new(name: &'static str) -> ContextKey<T> {
        ContextKey { name, _value: PhantomData }
    }
}

pub struct Context(Option<Arc<dyn Any + Send + Sync>>);

impl Context {
    pub fn value<T: Send + Sync + 'static>(&self, _key: &'static ContextKey<T>) -> Option<Arc<T>> {
        match &self.0 {
            Some(v) => v.clone().downcast::<T>().ok(),
            None => None,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Locale(String);

static CONTEXT_KEY: ContextKey<Locale> = ContextKey::new("contextKey(0)");

pub fn from_context(ctx: &Context) -> Locale {
    match ctx.value(&CONTEXT_KEY) {
        Some(locale) => (*locale).clone(),
        None => Locale::default(),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Life(u8);
static LIFE_KEY: ContextKey<Life> = ContextKey::new("life");

pub fn get_life(ctx: &Context) -> Life {
    if let Some(l) = ctx.value(&LIFE_KEY) {
        return *l;
    }
    Life(0)
}

fn main() {
    let c = Context(Some(Arc::new(Locale("en".into()))));
    println!("{:?} {}", from_context(&c), CONTEXT_KEY.name);
    let d = Context(Some(Arc::new(Life(3))));
    println!("{:?} {:?}", get_life(&d), from_context(&d));
}

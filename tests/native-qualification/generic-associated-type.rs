trait Lend {
    type Item<'a>
    where
        Self: 'a;
    fn get<'a>(&'a self) -> Self::Item<'a>;
}

struct Value(i32);

impl Lend for Value {
    type Item<'a> = &'a i32;

    fn get<'a>(&'a self) -> Self::Item<'a> {
        &self.0
    }
}

fn twice(value: i32) -> i32 {
    value * 2
}

fn main() {
    let value = Value(7);
    println!("{}", twice(*value.get()));
}

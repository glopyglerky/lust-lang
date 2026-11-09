// Generic parameter defaults on traits: an impl, a bound, a dyn type,
// and a qualified path may leave the defaulted input out; `Rhs = Self`
// takes the Self type of each use.
trait Get<T = i32> {
    fn get(&self) -> T;
}

trait Same<Rhs = Self> {
    fn same(&self, other: &Rhs) -> bool;
}

struct X(i32);

impl Get for X {
    fn get(&self) -> i32 {
        self.0
    }
}

impl Get<u8> for X {
    fn get(&self) -> u8 {
        8
    }
}

impl Same for X {
    fn same(&self, other: &X) -> bool {
        self.0 == other.0
    }
}

fn twice<A: Get>(a: &A) -> i32 {
    a.get() * 2
}

fn via_dyn(g: &dyn Get) -> i32 {
    g.get() + 1
}

fn main() {
    let x = X(5);
    println!("{} {} {}", twice(&x), via_dyn(&x), <X as Get<u8>>::get(&x));
    println!("{} {}", <X as Get>::get(&x), x.same(&X(5)));
}

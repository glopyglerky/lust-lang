// expect: L0110
// rustc E0393: the default of `Rhs` names `Self`, so `dyn Same` must write it.
trait Same<Rhs = Self> {
    fn same(&self, other: &Rhs) -> bool;
}

fn f(_x: &dyn Same) {}

fn main() {}

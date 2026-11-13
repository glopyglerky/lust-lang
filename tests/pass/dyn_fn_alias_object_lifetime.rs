// A `'_` arg of a type alias inside `dyn Fn*` sugar is the object's own
// late-bound lifetime, as when the alias is written out (ts_goport
// program_lookup.rs set_symlinks_from_resolutions; false reject from R11).
pub type Cb<'f, T> = dyn FnMut(&T, &str) + 'f;
struct K {
    n: u32,
}
impl K {
    fn note(&mut self, s: &str) {
        self.n += s.len() as u32;
    }
    fn set(&mut self, a: &dyn Fn(&mut Cb<'_, String>, Option<&u8>), b: &dyn Fn(&mut Cb<'_, u32>, Option<&u8>)) {
        a(
            &mut |r: &String, _m: &str| {
                self.note(r);
            },
            None,
        );
        b(
            &mut |_r: &u32, m: &str| {
                self.note(m);
            },
            None,
        );
    }
}
fn main() {
    let mut k = K { n: 0 };
    k.set(&|cb, _| cb(&"ab".to_string(), "x"), &|cb, _| cb(&1, "xyz"));
    println!("{}", k.n);
}

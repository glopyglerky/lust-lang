// Panics while an `array::IntoIter` of non-Copy elements is alive
// (fx-array-iter): the cleanup drops the elements not yet yielded,
// front to back, exactly once.
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    // A panic in a `map` closure mid-iteration.
    let r = std::panic::catch_unwind(|| {
        let v: Vec<u8> = [D(1), D(2), D(3), D(4)]
            .into_iter()
            .map(|d| {
                println!("map {}", d.0);
                if d.0 == 2 {
                    panic!("boom");
                }
                d.0
            })
            .collect();
        println!("never {:?}", v);
    });
    println!("caught {} -- done1", r.is_err());
    // A panic in a by-value `for` body.
    let r2 = std::panic::catch_unwind(|| {
        for d in [D(10), D(11), D(12), D(13)] {
            println!("for {}", d.0);
            if d.0 == 11 {
                panic!("boom2");
            }
        }
    });
    println!("caught2 {} -- done2", r2.is_err());
    // A panic while a partially consumed iterator is in scope.
    let r3 = std::panic::catch_unwind(|| {
        let mut it = [D(20), D(21), D(22)].into_iter();
        let a = it.next().unwrap();
        println!("a {}", a.0);
        panic!("boom3");
    });
    println!("caught3 {} -- done3", r3.is_err());
    // A panic inside `collect` over a zip of arrays.
    let r4 = std::panic::catch_unwind(|| {
        let v: Vec<(u8, u8)> = [D(1), D(2), D(3)]
            .into_iter()
            .map(|d| d.0)
            .zip([D(4), D(5), D(6)].into_iter().map(|d| {
                if d.0 == 5 {
                    panic!("boom4");
                }
                d.0
            }))
            .collect();
        println!("never {:?}", v);
    });
    println!("caught4 {} -- done4", r4.is_err());
}

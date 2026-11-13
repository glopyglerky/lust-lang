// A glob of a std module brings the items lust ships as std source
// overlays (iter::once, cell::OnceCell, sync::OnceLock), in modules,
// re-exports, and fn bodies; a local item still shadows a glob name.
mod g {
    use core::iter::*;
    use std::sync::*;
    pub fn run() {
        let v: Vec<i32> = once(1).chain(once_with(|| 2)).collect();
        let mut k = 0;
        let w: Vec<u8> = from_fn(|| {
            k += 1;
            if k < 3 { Some(k) } else { None }
        })
        .collect();
        let s: Vec<u32> = successors(Some(1u32), |x| if *x < 4 { Some(x * 2) } else { None }).collect();
        let ol: OnceLock<u8> = OnceLock::new();
        ol.set(1).unwrap();
        println!("{:?} {:?} {:?} {} {:?}", v, w, s, v.capacity(), ol.get());
    }
}
mod m {
    pub use std::cell::*;
}
mod n {
    use std::iter::*;
    pub fn once(x: i32) -> i32 {
        x * 10
    }
    pub fn first() -> i32 {
        let v: Vec<i32> = successors(Some(once(4)), |_| None).collect();
        v[0]
    }
}
fn main() {
    g::run();
    let oc: m::OnceCell<u16> = m::OnceCell::new();
    println!("{:?} {}", oc.get_or_init(|| 9), n::first());
    {
        use std::cell::*;
        let o: OnceCell<u8> = OnceCell::new();
        let c = Cell::new(2);
        println!("{:?} {}", o.get_or_init(|| 4), c.get());
    }
}

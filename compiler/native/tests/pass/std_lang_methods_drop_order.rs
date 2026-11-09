// Std source methods of Option and Result drop the values they do not
// return in core's order: the unused default, the other option, the
// error, the receiver's payload.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let a = Some(D("a")).map_or(D("default1"), |d| { println!("map {}", d.0); D("mapped") });
    println!("got {}", a.0);
    let b = None.map_or(D("default2"), |d: D| d);
    println!("got {}", b.0);
    let c = Some(D("c")).unwrap_or(D("unused"));
    println!("got {}", c.0);
    let d = Some(D("d")).and(Some(D("other")));
    println!("got {}", d.as_ref().map(|x| x.0).unwrap_or("none"));
    let e = Some(D("e")).or(Some(D("or-unused")));
    println!("got {}", e.as_ref().unwrap().0);
    let f = Some(D("f")).xor(Some(D("g")));
    println!("xor none {}", f.is_none());
    let g = Some(D("h")).filter(|x| x.0 == "zz");
    println!("filter none {}", g.is_none());
    let r: Result<D, D> = Err(D("err"));
    let h = r.unwrap_or(D("fallback"));
    println!("got {}", h.0);
    let r2: Result<D, D> = Ok(D("ok"));
    let i = r2.and_then(|x| { println!("and_then {}", x.0); Err::<D, D>(D("e2")) });
    println!("is_err {}", i.is_err());
    let mut z = Some(D("z1"));
    let old = z.replace(D("z2"));
    println!("old {} new {}", old.as_ref().unwrap().0, z.as_ref().unwrap().0);
    let t = z.take();
    println!("took {} left none {}", t.as_ref().unwrap().0, z.is_none());
    let mut w = Some(D("w1"));
    *w.insert(D("w2")) = D("w3");
    println!("end");
}

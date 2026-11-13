// Turbofish on the builtin Option and Result constructors fixes their
// type args (`_` stays inferred), on the constructor or on the type.
fn main() {
    let r = Ok::<u8, _>(1);
    let s: Result<u8, String> = r;
    let t = Result::<u8, ()>::Ok(3);
    let o = Option::<u8>::Some(3);
    let p = None::<u8>;
    let q: Option<u8> = Option::None;
    let n = None::<_>;
    let m: Option<i64> = n;
    let w = Some::<u16>(3);
    let v: Vec<Option<u8>> = vec![None::<u8>, Some(1)];
    println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", s, t, o, p, q, m, w, v);
}

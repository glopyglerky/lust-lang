// [T; N]::map (R22): order, drops, captures, empty arrays, borrows in the outputs.
#[derive(Debug)]
struct D(i32);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
struct S { next: [u32; 3] }
fn main() {
    let a = [1usize, 2, 3];
    let b = a.map(|x| x as u32 + 1);
    let v: Vec<usize> = vec![1];
    let lens = [v.len(), v.len(), v.len()];
    let s = S { next: lens.map(|l| l as u32) };
    println!("{:?} {:?} {:?}", b, s.next, a);
    let ds = [D(1), D(2), D(3)];
    let tag = D(100);
    let es = ds.map(move |d| { let _k = &tag; println!("see {}", d.0); D(d.0 * 10) });
    println!("built {:?}", es);
    let strs = ["a", "bb"].map(String::from);
    println!("{:?}", strs);
    let empty: [D; 0] = [];
    let e2: [u8; 0] = empty.map(|_| 1);
    println!("{:?}", e2);
    let names = ["x", "y"];
    let refs = names.map(|n| n.len());
    let mut cnt = 0;
    let c = [(); 4].map(|_| { cnt += 1; cnt });
    println!("{:?} {:?} {}", refs, c, cnt);
    let nested = [[1, 2], [3, 4]].map(|r| r.map(|x| x * x));
    println!("{:?}", nested);
    let own = vec![10, 20];
    let pick = [0usize, 1, 1].map(|i| &own[i]);
    println!("{:?}", pick);
    let tup = [(1, 'a'), (2, 'b')].map(|(n, ch)| format!("{n}{ch}"));
    println!("{:?}", tup);
}

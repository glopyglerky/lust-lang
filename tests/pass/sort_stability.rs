// Stable sorts keep equal elements in order; unstable sorts must match
// Rust's ipnsort placement of equal elements exactly. Large inputs go
// through driftsort's runs and merges and ipnsort's partitions.
fn lcg(s: &mut u64) -> u64 {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *s >> 33
}
fn main() {
    let mut seed = 7;
    for &n in &[17usize, 33, 64, 200, 1000] {
        let base: Vec<(u32, u32)> = (0..n as u32).map(|i| ((lcg(&mut seed) % 9) as u32, i)).collect();
        let mut st = base.clone();
        st.sort_by_key(|p| p.0);
        let mut un = base.clone();
        un.sort_unstable_by_key(|p| p.0);
        let mut un2 = base.clone();
        un2.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let ok = st.windows_ok();
        println!("{} {} {:?}", n, ok, &st[..8]);
        println!("{:?}", &un[..12]);
        println!("{:?}", &un2[n - 12..]);
        let mut wide: Vec<(u64, u64, u64)> = base.iter().map(|p| (p.0 as u64, p.1 as u64, 0)).collect();
        wide.sort_unstable_by_key(|p| p.0);
        println!("{:?}", &wide[..6]);
    }
}
trait WindowsOk {
    fn windows_ok(&self) -> bool;
}
impl WindowsOk for Vec<(u32, u32)> {
    fn windows_ok(&self) -> bool {
        let mut i = 1;
        while i < self.len() {
            let (a, b) = (self[i - 1], self[i]);
            if a.0 > b.0 || (a.0 == b.0 && a.1 > b.1) {
                return false;
            }
            i += 1;
        }
        true
    }
}

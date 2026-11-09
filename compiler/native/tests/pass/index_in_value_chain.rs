// An index place inside a value expression (`s[i..].chars()`) under a `&mut`
// use of that value (`.next()`) is read, not mutably borrowed (tsrust T10).
fn unit_at(s: &str, i: usize) -> Option<char> {
    s[i..].chars().next()
}
fn last_slash(s: &str) -> Option<usize> {
    s.as_bytes()[..s.len() - 1].iter().rposition(|&b| b == b'/')
}
fn before(s: &str, pos: usize) -> Option<char> {
    s[..pos].chars().next_back()
}
fn all_star(v: &Vec<u8>, i: usize) -> bool {
    v[i..].iter().all(|&x| x == b'*')
}
fn main() {
    println!("{:?} {:?} {:?} {}", unit_at("abc", 1), last_slash("a/b/c"), before("abc", 2), all_star(&b"ab**".to_vec(), 2));
}

// A long operator chain with side effects: operands must run left to right.
fn main() {
    let mut log = String::new();
    let mut n = 0i64;
    let total = {
        n += 1; log.push('a'); n
    } + {
        n += 1; log.push('b'); n
    } + {
        n += 1; log.push('c'); n
    };
    println!("{} {}", total, log);
}

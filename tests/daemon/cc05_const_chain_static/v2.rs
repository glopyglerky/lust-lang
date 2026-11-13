const fn cf() -> i32 { "bad" }
const C: i32 = cf() + 1;
static S: i32 = C * 10;
static TBL: [i32; 3] = [C, C + 1, S];
static R: &i32 = &S;
fn main() {
    println!("{} {} {:?} {}", C, S, TBL, *R);
}

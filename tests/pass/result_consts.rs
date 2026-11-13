// const and static Result/Option values, Option::Some / Option::None paths.
const CR: Result<i32, i32> = Ok(41);
static SR: Result<i32, i32> = Err(7);
const CN: Option<i32> = Some(3);
fn main() {
    println!("{:?} {:?} {:?}", CR, SR, CN);
    let x = Option::Some(5);
    let y: Option<i32> = Option::None;
    println!("{:?} {:?}", x, y);
}

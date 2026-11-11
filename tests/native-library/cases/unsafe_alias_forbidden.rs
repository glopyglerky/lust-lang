use left::unchecked_add as operation;

fn main() {
    println!("unsafe-alias={}", operation(25));
}

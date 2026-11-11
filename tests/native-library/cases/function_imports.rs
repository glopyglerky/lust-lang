use left::combine as combine_left;
use right::combine;

fn main() {
    println!("imports={},{}", combine_left(10, 1), combine(10, 1));
}

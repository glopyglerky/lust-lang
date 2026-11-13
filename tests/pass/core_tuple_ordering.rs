fn main() {
    println!("{}", (1, 2) < (1, 3));
    println!("{}", (1, 3) < (1, 2));
    println!("{}", (2, 0) > (1, 9));
    println!("{} {}", (1, 2) <= (1, 2), (1, 2) >= (1, 2));
    println!("{} {}", (1, 2) <= (1, 1), (1, 2) >= (1, 3));
    println!("{}", (1, (2, 3)) < (1, (2, 4)));
    println!("{}", ("a", 1) < ("b", 0));
    println!("{}", ("a", 2) > ("a", 1));
    let a = (1, String::from("x"));
    let b = (1, String::from("y"));
    println!("{}", a < b);
    println!("{}", (true, 0) > (false, 9));
    println!("{}", (1.5, 0) < (2.5, 0));
    println!("{}", (1.5, 2) > (1.5, 1));
    let t = (3, 1);
    let u = (2, 9);
    println!("{} {}", t > u, &t < &u);
    println!("{}", () < ());
    println!("{}", () <= ());
    println!("{} {}", (5,) < (6,), (5,) > (6,));
    println!("{}", (0, 'a') < (0, 'b'));
}

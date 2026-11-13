// Calls in every position the Call terminator now ends a block in:
// nested arguments, conditions, loop heads, match arms, breaks, and
// overwrites of locals that own a value (the old value must drop once).

fn twice(x: i64) -> i64 {
    x * 2
}

fn bump(x: i64) -> i64 {
    x + 1
}

fn fib(n: i64) -> i64 {
    if n < 2 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn label(n: i64) -> String {
    if n % 2 == 0 {
        String::from("even")
    } else {
        String::from("odd")
    }
}

struct Bag {
    items: Vec<i64>,
}

impl Bag {
    fn total(&self) -> i64 {
        let mut sum = 0;
        let mut i = 0;
        while i < self.items.len() {
            sum += twice(self.items[i]);
            i += 1;
        }
        sum
    }
    fn add(&mut self, v: i64) {
        self.items.push(bump(v));
    }
}

fn main() {
    // Nested calls as arguments.
    println!("{}", twice(bump(twice(10))));

    // Recursion: two calls in one expression.
    println!("{}", fib(10));

    // A call result overwrites an owned local; the old String drops here.
    let mut s = label(1);
    println!("{}", s);
    s = label(2);
    println!("{}", s);
    s += label(3).as_str();
    println!("{}", s);

    // Calls in a condition and in both arms.
    let mut bag = Bag { items: Vec::new() };
    let mut n = 0;
    while bump(n) <= 4 {
        bag.add(n);
        n += 1;
    }
    if bag.total() > 10 {
        println!("big {}", bag.total());
    } else {
        println!("small {}", bag.total());
    }

    // A call in a match arm and as a loop break value.
    let word = match bump(2) {
        3 => label(4),
        _ => label(5),
    };
    println!("{}", word);
    let mut i = 0;
    let stop = loop {
        i += 1;
        if i == 3 {
            break twice(i);
        }
    };
    println!("{}", stop);

    // Builtin calls in a for head and a method chain on a temporary.
    let mut digits = 0;
    for c in "a1b2c3".chars() {
        if c.is_ascii_digit() {
            digits += bump(0);
        }
    }
    println!("{} {}", digits, label(9).len());
}

// Block-like match arm bodies need no comma (labeled block: ts-rust checker_p33).
fn f(x: u32, c: bool) -> u32 {
    match x {
        0 => if c { 1 } else { 2 }
        1 => match c { true => 3, false => 4 }
        3 => loop { break 6 }
        4 => 'l: loop { break 'l 7 }
        5 => const { 8 }
        _ => { 9 }
    }
}
fn main() { for i in 0..6 { print!("{} ", f(i, i % 2 == 0)); } println!(); }

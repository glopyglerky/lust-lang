// Widths and precisions from arguments (N$, name$, captured, .*) and named arguments.
fn main() {
    let s = format!("{:1$}", "a", 65535usize);
    println!("{}", s.len());
    let p = format!("{:.*}", 2, 1.23456);
    println!("{}", p);
    let w = 5; let x = 3;
    println!("[{x:w$}] [{:>1$}] [{:.prec$}] [{2:>width$.prec$}]", 7, 4, 2.5, prec = 2, width = w);
    println!("[{:>1$}] [{:*^2$.3$}]", "ab", 6, 9, 1);
    println!("[{:+.*}] [{:#0w$x}]", 3, -1.5f64, 255, w = 8);
    let name_width = 3; let v = "x";
    println!("{:<name_width$}|{:>v$}|", "ab", "c", name_width = name_width + 1, v = 2);
}

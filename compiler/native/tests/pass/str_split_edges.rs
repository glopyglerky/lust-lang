// Split family edge cases (W-STD-B): empty haystacks and needles, lone
// separators, counts, char sets by slice and array ref, alternating ends.
fn show(name: &str, v: Vec<&str>) {
    println!("{name}: {v:?}");
}
fn main() {
    show("lines crlf", "\r\n".lines().collect());
    show("lines nl", "\n".lines().collect());
    show("lines empty", "".lines().collect());
    show("lines lone cr", "a\rb\r".lines().collect());
    show("ws empty", "   ".split_whitespace().collect());
    show("incl empty", "".split_inclusive('x').collect());
    show("incl rev", "a.b.".split_inclusive('.').rev().collect());
    show("rsplitn 1", "a,b".rsplitn(1, ',').collect());
    show("splitn big", "a,b".splitn(9, ',').collect());
    show("rsplit empty needle", "ab".rsplit("").collect());
    show("splitn empty needle", "ab".splitn(2, "").collect());
    show("term empty", "".split_terminator(',').collect());
    show("term only sep", ",".split_terminator(',').collect());
    show("rterm only sep", ",,".rsplit_terminator(',').collect());
    show("set slice", "a-b+c".split(&['-', '+'][..]).collect());
    show("set ref arr", "a-b+c".split(&['-', '+']).collect());
    show("needle longer", "ab".split("abc").collect());
    show("needle at end", "abc".split("bc").collect());
    let mut it = "a,b,c,d".split(',');
    let mut out = Vec::new();
    while let (Some(x), Some(y)) = (it.next(), it.next_back()) {
        out.push(x);
        out.push(y);
    }
    show("alternate", out);
    println!("{:?} {:?}", "".split_once(','), ",".rsplit_once(','));
}

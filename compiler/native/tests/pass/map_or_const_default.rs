// A constant at a type param's position (`map_or("", f)`, `pick(x, "")`)
// binds after the other arguments and the return: the `'static` field
// read does not tie the argument to `'static` (ts-rust effect/rule.rs).
pub struct Rule { pub name: &'static str, pub codes: &'static [i32] }
pub fn code_to_rule_name(rules: &[&'static Rule], code: i32) -> &'static str {
    rules.iter().find(|r| r.codes.contains(&code)).map_or("", |r| r.name)
}
fn name_of<'a>(r: Option<&'a Rule>) -> &'static str { r.map_or("none", |r| r.name) }
fn pick<T>(a: T, _b: T) -> T { a }
fn first<'a>(r: &'a Rule) -> &'static str { pick(r.name, "") }
fn main() {
    static R: Rule = Rule { name: "x", codes: &[1, 2] };
    let r: &'static Rule = &R;
    println!("{} [{}]", code_to_rule_name(&[r], 2), code_to_rule_name(&[r], 3));
    let local = Rule { name: "local", codes: &[] };
    println!("{} {} {}", name_of(Some(&local)), name_of(None), first(&local));
}

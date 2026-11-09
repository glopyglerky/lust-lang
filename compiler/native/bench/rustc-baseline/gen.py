# Generates lust-subset Rust programs for rustc baseline timing.
# Usage: python3 gen.py <shape: plain|idiomatic> <units> <variant> > out.rs
# `variant` changes one constant in one function body, to simulate a one-line edit.
import sys

shape, units, variant = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
out = ["use std::collections::HashMap;"]

def plain(i):
    prev = f"run{i-1}(t % 7)" if i else "0"
    return f"""
#[derive(Debug, Clone, PartialEq)]
enum Kind{i} {{ A(i64), B {{ x: i64, y: i64 }}, C }}
#[derive(Debug, Clone)]
struct S{i} {{ id: i64, name: String, kind: Kind{i}, items: Vec<i64> }}
fn make{i}(n: i64) -> S{i} {{
    let kind = if n % 3 == 0 {{ Kind{i}::A(n) }} else if n % 3 == 1 {{ Kind{i}::B {{ x: n, y: n * 2 }} }} else {{ Kind{i}::C }};
    S{i} {{ id: n, name: format!("s{{}}", n), kind, items: vec![n, n + 1, n + 2] }}
}}
fn score{i}(s: &S{i}) -> i64 {{
    let mut total = s.id + s.name.len() as i64;
    for v in &s.items {{ total += *v; }}
    match &s.kind {{ Kind{i}::A(v) => total + v, Kind{i}::B {{ x, y }} => total + x * y, Kind{i}::C => total - 1 }}
}}
fn run{i}(n: i64) -> i64 {{ let s = make{i}(n); let t = score{i}(&s) + {variant if i == units // 2 else 0}; if t > 100 {{ t - {prev} }} else {{ t }} }}"""

def idiomatic(i):
    return f"""
#[derive(Debug, Clone)]
struct R{i} {{ name: String, value: u32 }}
trait T{i} {{ fn val(&self) -> u32; }}
impl T{i} for R{i} {{ fn val(&self) -> u32 {{ self.value + {variant if i == units // 2 else 0} }} }}
fn parse{i}(line: &str) -> Result<Vec<R{i}>, String> {{
    line.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).map(|p| {{
        let (a, b) = p.split_once(':').ok_or(format!("bad {{}}", p))?;
        Ok(R{i} {{ name: a.to_string(), value: b.parse::<u32>().map_err(|e| e.to_string())? }})
    }}).collect()
}}
fn sum{i}<X: T{i}>(xs: &[X]) -> u32 {{ xs.iter().map(|x| x.val()).sum() }}
fn group{i}(rs: &[R{i}]) -> HashMap<String, u32> {{
    let mut m = HashMap::new();
    for r in rs {{ *m.entry(r.name.clone()).or_insert(0) += r.value; }}
    m
}}
fn run{i}(s: &str) -> u32 {{ match parse{i}(s) {{ Ok(rs) => sum{i}(&rs) + group{i}(&rs).len() as u32, Err(_) => 0 }} }}"""

gen = plain if shape == "plain" else idiomatic
for i in range(units):
    out.append(gen(i))
if shape == "plain":
    # Each run{i} calls run{i-1}, so calling a sample keeps every unit reachable.
    calls = " + ".join(f"run{i}(n)" for i in range(0, units, max(1, units // 50)))
    out.append(f"fn main() {{ let n = std::env::args().count() as i64; let mut acc = 0i64; for i in 0..{units} {{ acc = acc.wrapping_add(i); }} println!(\"{{}}\", acc + {calls}); }}")
else:
    calls = "\n".join(f"    acc += run{i}(input);" for i in range(units))
    out.append(f"fn main() {{\n    let input = \"a:1, b:2, a:3\";\n    let mut acc = 0u32;\n{calls}\n    println!(\"{{}}\", acc);\n}}")
print("\n".join(out))

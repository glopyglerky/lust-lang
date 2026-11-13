// Many functions over one glue-heavy type, plus a rare type used by two functions.
// A cold build makes several function units here: Bag glue (drop, clone, eq, debug) is
// reached by every unit and lives in the shared glue unit; Rare glue is reached by one
// unit and is static there. Both paths must give the same output as rustc.
#[derive(Debug, Clone, PartialEq)]
struct Bag {
    name: String,
    vals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct Rare {
    s: String,
    n: i64,
}

fn rare_len(r: Rare) -> usize {
    let c = r.clone();
    if c == r {
        c.s.len()
    } else {
        0
    }
}

fn rare_sum(r: &Rare) -> i64 {
    r.n + r.s.len() as i64
}

fn bag_new(name: &str, n: usize) -> Bag {
    let mut vals = Vec::new();
    let mut i = 0;
    while i < n {
        vals.push(format!("v{}", i));
        i += 1;
    }
    Bag { name: name.to_string(), vals }
}

fn f000(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f001(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f002(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f003(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f004(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f005(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f006(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f007(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f008(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f009(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f010(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f011(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f012(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f013(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f014(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f015(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f016(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f017(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f018(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f019(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f020(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f021(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f022(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f023(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f024(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f025(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f026(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f027(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f028(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f029(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f030(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f031(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f032(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f033(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f034(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f035(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f036(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f037(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f038(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f039(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f040(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f041(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f042(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f043(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f044(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f045(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f046(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f047(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f048(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f049(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f050(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f051(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f052(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f053(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f054(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f055(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f056(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f057(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f058(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f059(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f060(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f061(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f062(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f063(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f064(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f065(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f066(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f067(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f068(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f069(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f070(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f071(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f072(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f073(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f074(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f075(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f076(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f077(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f078(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f079(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f080(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f081(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f082(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f083(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f084(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f085(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f086(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f087(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f088(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f089(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f090(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f091(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f092(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f093(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f094(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f095(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f096(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f097(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f098(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f099(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f100(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f101(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f102(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f103(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f104(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f105(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f106(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f107(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f108(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f109(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f110(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f111(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f112(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f113(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f114(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f115(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f116(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f117(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f118(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f119(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f120(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f121(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f122(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f123(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f124(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f125(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f126(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f127(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f128(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f129(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f130(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f131(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f132(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f133(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f134(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f135(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f136(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f137(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f138(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f139(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f140(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f141(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f142(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f143(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f144(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f145(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f146(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f147(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f148(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn f149(b: Bag) -> usize {
    let c = b.clone();
    let same = if c == b { 1 } else { 0 };
    c.vals.len() + c.name.len() + same
}

fn main() {
    let bag = bag_new("shared", 3);
    let total = f000(bag.clone()) + f001(bag.clone()) + f063(bag.clone()) + f064(bag.clone())
        + f127(bag.clone()) + f128(bag.clone()) + f149(bag.clone());
    let rare = Rare { s: String::from("rare!"), n: 40 };
    println!("{} {} {}", total, rare_len(rare.clone()), rare_sum(&rare));
    println!("{:?}", bag);
    println!("{:?}", rare);
}

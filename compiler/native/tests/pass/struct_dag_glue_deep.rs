// A struct DAG 20 levels deep: each level holds two fields of the next level and an Rc of it
// (3^20 paths, 21 types). The derive glue checks (Default, Clone, PartialEq, PartialOrd, Debug)
// and the deep fingerprints walk each type once, so this checks fast (it took minutes when they
// walked every path). Hash is left out: lust's derive bound check overflows the solver depth on
// this DAG past 12 levels (a separate false reject).
use std::rc::Rc;

#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S20 { pub a: S19, pub b: S19, pub r: Rc<S19> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S19 { pub a: S18, pub b: S18, pub r: Rc<S18> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S18 { pub a: S17, pub b: S17, pub r: Rc<S17> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S17 { pub a: S16, pub b: S16, pub r: Rc<S16> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S16 { pub a: S15, pub b: S15, pub r: Rc<S15> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S15 { pub a: S14, pub b: S14, pub r: Rc<S14> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S14 { pub a: S13, pub b: S13, pub r: Rc<S13> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S13 { pub a: S12, pub b: S12, pub r: Rc<S12> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S12 { pub a: S11, pub b: S11, pub r: Rc<S11> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S11 { pub a: S10, pub b: S10, pub r: Rc<S10> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S10 { pub a: S9, pub b: S9, pub r: Rc<S9> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S9 { pub a: S8, pub b: S8, pub r: Rc<S8> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S8 { pub a: S7, pub b: S7, pub r: Rc<S7> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S7 { pub a: S6, pub b: S6, pub r: Rc<S6> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S6 { pub a: S5, pub b: S5, pub r: Rc<S5> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S5 { pub a: S4, pub b: S4, pub r: Rc<S4> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S4 { pub a: S3, pub b: S3, pub r: Rc<S3> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S3 { pub a: S2, pub b: S2, pub r: Rc<S2> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S2 { pub a: S1, pub b: S1, pub r: Rc<S1> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S1 { pub a: S0, pub b: S0, pub r: Rc<S0> }
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct S0 { pub x: u32 }

pub struct Top { pub s: S20, pub n: u32 }

impl Top {
    pub fn bump(&mut self, v: u32) -> u32 {
        self.n += v;
        self.n
    }
}

// Never called: building S20 makes 3^20 values. The checks still run on its body.
fn build() -> u64 {
    let t = Top { s: S20::default(), n: 0 };
    let u = t.s.clone();
    println!("{:?} {}", t.s.a.a.a, t.s < u);
    (t.s == u) as u64
}

pub struct Small { pub s: S1, pub n: u32 }

impl Small {
    pub fn bump(&mut self, v: u32) -> u32 {
        self.n += v;
        self.n
    }
}

fn main() {
    let mut t = Small { s: S1::default(), n: 0 };
    if std::env::args().count() > 100 {
        println!("{}", build());
    }
    println!("{} {:?}", t.bump(3), t.s);
}

// expect: L0306
// rustc E0597: the `'static` bound sits 70 supertraits down; the search for it
// must not stop early and answer "not 'static".
trait S0: 'static {}
trait S1: S0 {}
trait S2: S1 {}
trait S3: S2 {}
trait S4: S3 {}
trait S5: S4 {}
trait S6: S5 {}
trait S7: S6 {}
trait S8: S7 {}
trait S9: S8 {}
trait S10: S9 {}
trait S11: S10 {}
trait S12: S11 {}
trait S13: S12 {}
trait S14: S13 {}
trait S15: S14 {}
trait S16: S15 {}
trait S17: S16 {}
trait S18: S17 {}
trait S19: S18 {}
trait S20: S19 {}
trait S21: S20 {}
trait S22: S21 {}
trait S23: S22 {}
trait S24: S23 {}
trait S25: S24 {}
trait S26: S25 {}
trait S27: S26 {}
trait S28: S27 {}
trait S29: S28 {}
trait S30: S29 {}
trait S31: S30 {}
trait S32: S31 {}
trait S33: S32 {}
trait S34: S33 {}
trait S35: S34 {}
trait S36: S35 {}
trait S37: S36 {}
trait S38: S37 {}
trait S39: S38 {}
trait S40: S39 {}
trait S41: S40 {}
trait S42: S41 {}
trait S43: S42 {}
trait S44: S43 {}
trait S45: S44 {}
trait S46: S45 {}
trait S47: S46 {}
trait S48: S47 {}
trait S49: S48 {}
trait S50: S49 {}
trait S51: S50 {}
trait S52: S51 {}
trait S53: S52 {}
trait S54: S53 {}
trait S55: S54 {}
trait S56: S55 {}
trait S57: S56 {}
trait S58: S57 {}
trait S59: S58 {}
trait S60: S59 {}
trait S61: S60 {}
trait S62: S61 {}
trait S63: S62 {}
trait S64: S63 {}
trait S65: S64 {}
trait S66: S65 {}
trait S67: S66 {}
trait S68: S67 {}
trait S69: S68 {}
trait Read: S69 { fn read(&self) -> u32; }
struct W<X> { x: X }
impl<X: 'static> S0 for W<X> {}
impl<X: 'static> S1 for W<X> {}
impl<X: 'static> S2 for W<X> {}
impl<X: 'static> S3 for W<X> {}
impl<X: 'static> S4 for W<X> {}
impl<X: 'static> S5 for W<X> {}
impl<X: 'static> S6 for W<X> {}
impl<X: 'static> S7 for W<X> {}
impl<X: 'static> S8 for W<X> {}
impl<X: 'static> S9 for W<X> {}
impl<X: 'static> S10 for W<X> {}
impl<X: 'static> S11 for W<X> {}
impl<X: 'static> S12 for W<X> {}
impl<X: 'static> S13 for W<X> {}
impl<X: 'static> S14 for W<X> {}
impl<X: 'static> S15 for W<X> {}
impl<X: 'static> S16 for W<X> {}
impl<X: 'static> S17 for W<X> {}
impl<X: 'static> S18 for W<X> {}
impl<X: 'static> S19 for W<X> {}
impl<X: 'static> S20 for W<X> {}
impl<X: 'static> S21 for W<X> {}
impl<X: 'static> S22 for W<X> {}
impl<X: 'static> S23 for W<X> {}
impl<X: 'static> S24 for W<X> {}
impl<X: 'static> S25 for W<X> {}
impl<X: 'static> S26 for W<X> {}
impl<X: 'static> S27 for W<X> {}
impl<X: 'static> S28 for W<X> {}
impl<X: 'static> S29 for W<X> {}
impl<X: 'static> S30 for W<X> {}
impl<X: 'static> S31 for W<X> {}
impl<X: 'static> S32 for W<X> {}
impl<X: 'static> S33 for W<X> {}
impl<X: 'static> S34 for W<X> {}
impl<X: 'static> S35 for W<X> {}
impl<X: 'static> S36 for W<X> {}
impl<X: 'static> S37 for W<X> {}
impl<X: 'static> S38 for W<X> {}
impl<X: 'static> S39 for W<X> {}
impl<X: 'static> S40 for W<X> {}
impl<X: 'static> S41 for W<X> {}
impl<X: 'static> S42 for W<X> {}
impl<X: 'static> S43 for W<X> {}
impl<X: 'static> S44 for W<X> {}
impl<X: 'static> S45 for W<X> {}
impl<X: 'static> S46 for W<X> {}
impl<X: 'static> S47 for W<X> {}
impl<X: 'static> S48 for W<X> {}
impl<X: 'static> S49 for W<X> {}
impl<X: 'static> S50 for W<X> {}
impl<X: 'static> S51 for W<X> {}
impl<X: 'static> S52 for W<X> {}
impl<X: 'static> S53 for W<X> {}
impl<X: 'static> S54 for W<X> {}
impl<X: 'static> S55 for W<X> {}
impl<X: 'static> S56 for W<X> {}
impl<X: 'static> S57 for W<X> {}
impl<X: 'static> S58 for W<X> {}
impl<X: 'static> S59 for W<X> {}
impl<X: 'static> S60 for W<X> {}
impl<X: 'static> S61 for W<X> {}
impl<X: 'static> S62 for W<X> {}
impl<X: 'static> S63 for W<X> {}
impl<X: 'static> S64 for W<X> {}
impl<X: 'static> S65 for W<X> {}
impl<X: 'static> S66 for W<X> {}
impl<X: 'static> S67 for W<X> {}
impl<X: 'static> S68 for W<X> {}
impl<X: 'static> S69 for W<X> {}
impl<X: 'static> Read for W<X> { fn read(&self) -> u32 { 4 } }
fn main() { let n = 7u32; let w = W { x: &n }; let d: &dyn Read = &w; println!("{}", d.read()); }

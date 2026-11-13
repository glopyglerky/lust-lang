// macro: payload_codec
// source: crates/ts_goport/src/frontend/parser/lib_parse_snapshot.rs:899
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
pub mod astdata {
    #[derive(Debug, Default)]
    pub struct PairData { pub left: u32, pub right: u32 }
    #[derive(Debug, Default)]
    pub struct NameData { pub text: String, pub flags: u32 }
}
#[derive(Debug)]
pub enum NodeData {
    Pair(Box<astdata::PairData>),
    Named(Box<astdata::NameData>),
}
pub struct Decoder<'a> { pub nums: &'a [u32], pub pos: usize }
impl Decoder<'_> {
    pub fn num(&mut self) -> Option<u32> { let v = *self.nums.get(self.pos)?; self.pos += 1; Some(v) }
    pub fn string(&mut self) -> Option<String> { Some(format!("s{}", self.num()?)) }
}
// Only the #[cfg(test)] half of the macro output names Encoder.
pub struct Encoder { pub out: Vec<u32> }
impl Encoder {
    pub fn u8(&mut self, v: u8) { self.out.push(u32::from(v)); }
    pub fn num(&mut self, v: &u32) -> Result<(), String> { self.out.push(*v); Ok(()) }
    pub fn string(&mut self, v: &String) -> Result<(), String> { self.out.push(v.len() as u32); Ok(()) }
}

macro_rules! payload_codec {
    ($($variant_tag:literal $variant:ident($data:ident) {
        $($field:ident: $codec:ident),* $(,)?
    })*) => {
        /// The node data with variant tag `variant_tag`, from `dec`.
        fn decode_payload(dec: &mut Decoder<'_>, variant_tag: u8) -> Option<NodeData> {
            Some(match variant_tag {
                $($variant_tag => NodeData::$variant(Box::new(crate::astdata::$data {
                    $($field: dec.$codec()?,)*
                })),)*
                _ => return None,
            })
        }

        /// Writes the variant tag and the fields of `payload`.
        #[cfg(test)]
        fn encode_payload(enc: &mut Encoder, payload: &NodeData) -> Result<(), String> {
            match payload {
                $(NodeData::$variant(payload) => {
                    let crate::astdata::$data { $($field),* } = &**payload;
                    enc.u8($variant_tag);
                    $(enc.$codec($field)?;)*
                })*
            }
            Ok(())
        }
    };
}

payload_codec! {
    0 Pair(PairData) { left: num, right: num }
    1 Named(NameData) { text: string, flags: num }
}

fn main() {
    // The macro names crate::astdata::$data, so $data here is the full
    // astdata path segment the invocation in ts-rust writes as a bare
    // data type name under the same module layout.
    let mut dec = Decoder { nums: &[3, 4], pos: 0 };
    println!("pair={:?}", decode_payload(&mut dec, 0));
    let mut dec = Decoder { nums: &[9, 1], pos: 0 };
    println!("named={:?}", decode_payload(&mut dec, 1));
    let mut dec = Decoder { nums: &[], pos: 0 };
    println!("bad={:?}", decode_payload(&mut dec, 7).map(|_| "some"));
}

// `Self::Target` in a DerefMut impl and `Self::Item` in a DoubleEndedIterator impl name the
// binding of another impl (the supertrait's); its regions follow that impl's header (ts-rust
// lib_parse_snapshot.rs PayloadDecoder).
struct Decoder<'a> {
    r: &'a [u8],
    pos: usize,
    ids: Vec<u32>,
}

impl<'a> Decoder<'a> {
    fn flag(&mut self) -> Option<bool> {
        let b = *self.r.get(self.pos)?;
        self.pos += 1;
        Some(b != 0)
    }
    fn num(&mut self) -> Option<u32> {
        let b = *self.r.get(self.pos)?;
        self.pos += 1;
        Some(b as u32)
    }
    fn list_parts(&mut self) -> Option<(usize, bool)> {
        let n = self.num()? as usize;
        self.ids.clear();
        for _ in 0..n {
            let v = self.num()?;
            self.ids.push(v);
        }
        Some((n, self.flag()?))
    }
}

struct Table<'l> {
    names: Vec<&'l str>,
    out: Vec<u32>,
}

impl Table<'_> {
    fn push(&mut self, range: usize, ids: &[u32], comma: bool, flags: u32) -> usize {
        self.out.extend_from_slice(ids);
        self.out.push(range as u32 + comma as u32 + flags + self.names.len() as u32);
        self.out.len()
    }
}

struct PayloadDecoder<'d, 'a, 't, 'l> {
    dec: &'d mut Decoder<'a>,
    lists: &'t mut Table<'l>,
}

impl<'a> std::ops::Deref for PayloadDecoder<'_, 'a, '_, '_> {
    type Target = Decoder<'a>;
    fn deref(&self) -> &Decoder<'a> {
        self.dec
    }
}

impl std::ops::DerefMut for PayloadDecoder<'_, '_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dec
    }
}

impl PayloadDecoder<'_, '_, '_, '_> {
    fn list(&mut self) -> Option<usize> {
        let (range, comma) = self.dec.list_parts()?;
        Some(self.lists.push(range, &self.dec.ids, comma, Default::default()))
    }
    fn olist(&mut self) -> Option<Option<usize>> {
        Some(if self.flag()? { Some(self.list()?) } else { None })
    }
    fn mods(&mut self) -> Option<Option<usize>> {
        if !self.flag()? {
            return Some(None);
        }
        let (range, comma) = self.dec.list_parts()?;
        let flags = self.num()?;
        Some(Some(self.lists.push(range, &self.dec.ids, comma, flags)))
    }
}

struct Data { a: Option<usize>, b: Option<usize>, c: Option<usize>, d: bool }

fn decode(dec: &mut PayloadDecoder<'_, '_, '_, '_>) -> Option<Data> {
    Some(Data { a: Some(dec.list()?), b: dec.olist()?, c: dec.mods()?, d: dec.flag()? })
}

fn main_decoder() {
    let bytes = [2u8, 7, 8, 1, 1, 1, 9, 1, 0, 1, 1, 3, 4, 0, 5, 1];
    let mut d = Decoder { r: &bytes, pos: 0, ids: Vec::new() };
    let s = String::from("x");
    let mut t = Table { names: vec![s.as_str()], out: Vec::new() };
    let data = decode(&mut PayloadDecoder { dec: &mut d, lists: &mut t });
    match data {
        Some(x) => println!("{:?} {:?} {:?} {}", x.a, x.b, x.c, x.d),
        None => println!("none"),
    }
    println!("{:?}", t.out);
}
struct It<'a> { s: &'a [u8] }
impl<'a> Iterator for It<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<&'a u8> {
        let (f, r) = self.s.split_first()?;
        self.s = r;
        Some(f)
    }
}
impl DoubleEndedIterator for It<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (l, r) = self.s.split_last()?;
        self.s = r;
        Some(l)
    }
}
fn main_iter() {
    let v = vec![1u8, 2, 3];
    let mut it = It { s: &v };
    let a = it.next_back();
    let b = it.next();
    println!("{:?} {:?}", a, b);
}
fn main() {
    main_decoder();
    main_iter();
}

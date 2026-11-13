// A region-only edit of an `Fn` bound's input (`&'static Message` to an
// elided, higher-ranked `&Message`): the unchanged `main` must recheck
// (W-BORROW, `RegionSig::fn_bounds` in the region-sig fingerprint).
struct Message {
    text: &'static str,
}

static M: Message = Message { text: "hi" };

fn report<F: FnMut(&'static Message)>(mut f: F) {
    f(&M);
}

fn main() {
    let mut keep: Vec<&'static Message> = Vec::new();
    report(|m: &'static Message| keep.push(m));
    println!("{}", keep[0].text);
}

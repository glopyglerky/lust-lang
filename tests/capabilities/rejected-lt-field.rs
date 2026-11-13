// capability: rejected: references in struct and enum fields without a lifetime (E0106)
// expect: L0201

struct S { r: &u32 }

fn main() {}

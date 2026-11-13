pub enum Fault { Local }
pub struct Bundle;

pub fn result() -> Result<(), crate::Fault> { Err(crate::Fault::Code(42)) }

pub fn classify(value: crate::Fault) -> u8 {
    match value {
        crate::Fault::Missing => 0,
        crate::Fault::Code(value) => value,
    }
}

pub fn associated() -> u8 { crate::Fault::missing().code() }

pub fn generic_shadow<Bundle>() -> crate::Bundle {
    crate::Bundle { values: [20, 22] }
}

pub fn typed() -> crate::Bundle<u16, 3> {
    crate::Bundle { values: [10, 20, 12] }
}

pub fn block_shadow() -> crate::Bundle {
    struct Bundle;
    crate::Bundle { values: [19, 23] }
}

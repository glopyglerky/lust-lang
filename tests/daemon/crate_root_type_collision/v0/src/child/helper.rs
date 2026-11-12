use super::*;

fn result() -> Result<(), crate::Fault> { Err(crate::Fault::Code(42)) }

pub fn answer() -> u8 {
    let cell: crate::Bundle = crate::Bundle { values: [20, 22] };
    match result() {
        Err(crate::Fault::Code(value)) => {
            assert_eq!(crate::Fault::Code(value).code(), value);
            assert_eq!(cell.values[0] + cell.values[1], value);
            value
        }
        Ok(()) => 0,
    }
}

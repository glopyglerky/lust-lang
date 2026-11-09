// Selection with fresh trait inputs is ambiguous when one Self
// has an impl per input; the written argument fixes the input
// through the trait method signature, in plain and generic calls.
trait Convert<In> {
    type Out;
    fn convert(&self, input: In) -> Self::Out;
}

struct Doubler;

impl Convert<u8> for Doubler {
    type Out = u16;
    fn convert(&self, input: u8) -> u16 {
        input as u16 * 2
    }
}

impl Convert<u16> for Doubler {
    type Out = u32;
    fn convert(&self, input: u16) -> u32 {
        input as u32 * 2
    }
}

fn use_it<C: Convert<u8, Out = u16>>(c: &C, value: u8) -> u16 {
    c.convert(value) + 1
}

fn main() {
    let d = Doubler;
    println!("{} {}", use_it(&d, 5), d.convert(6u16));
}

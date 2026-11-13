use base::base_add;
use mid::base_add as via_add;
use mid::Thing;

fn main() {
    let t: Thing = base::inner::make(1);
    let k = match base::Kind::B(7) {
        base::Kind::A => 0,
        base::Kind::B(x) => x,
    };
    println!(
        "{} {} {} {} {} {} {}",
        t.v,
        mid::via(2),
        base_add!(1),
        via_add!(1),
        base::base_add!(0),
        base::inner::call_secret(),
        k + mid::kind_val(mid::Kind::A)
    );
}

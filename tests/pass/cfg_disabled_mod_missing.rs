#[cfg(any())]
mod missing_a_cfg_parse;

#[cfg_attr(all(), cfg(any()))]
mod missing_b_cfg_parse;

#[cfg(any())]
mod m {
    pub mod missing_c_cfg_parse;
}

fn main() {
    println!("ok");
}

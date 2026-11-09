// Rust oracle for the float fuzz gate. Reads cases from stdin (one per
// line, as gen.c emits them), writes expected output to stdout. The C
// harness must byte-match this program's output. Build: rustc -O.
use std::io::{BufRead, BufWriter, Write};

// The spec table for spec64/spec32 modes. tests/float/harness.c holds
// the same list as lust_float_spec values; keep the two in sync.
//  0 {:.0}      1 {:.1}      2 {:.2}      3 {:.5}      4 {:.10}
//  5 {:.17}     6 {:.30}     7 {:.2?}     8 {:.0?}     9 {:.0e}
// 10 {:.2e}    11 {:.6E}    12 {:.20e}   13 {:12}     14 {:<12}
// 15 {:^12}    16 {:^13}    17 {:012}    18 {:+012}   19 {:+12}
// 20 {:012.3}  21 {:<012.2} 22 {:*>12}   23 {:*<12}   24 {:*^13}
// 25 {:0>12}   26 {:0<12}   27 {:#}      28 {:#.2}    29 {:#?}
// 30 {:12.4e}  31 {:+012.2e} 32 {:.50}   33 {:.100e}  34 {:é>10}
// 35 {:5}      36 {:.500}   37 {:^012}   38 {:+}      39 {:+?}
// 40 {:+e}     41 {:012e}   42 {:12.0}   43 {:^+12}
macro_rules! spec_format {
    ($v:expr, $idx:expr) => {
        match $idx {
            0 => format!("{:.0}", $v),
            1 => format!("{:.1}", $v),
            2 => format!("{:.2}", $v),
            3 => format!("{:.5}", $v),
            4 => format!("{:.10}", $v),
            5 => format!("{:.17}", $v),
            6 => format!("{:.30}", $v),
            7 => format!("{:.2?}", $v),
            8 => format!("{:.0?}", $v),
            9 => format!("{:.0e}", $v),
            10 => format!("{:.2e}", $v),
            11 => format!("{:.6E}", $v),
            12 => format!("{:.20e}", $v),
            13 => format!("{:12}", $v),
            14 => format!("{:<12}", $v),
            15 => format!("{:^12}", $v),
            16 => format!("{:^13}", $v),
            17 => format!("{:012}", $v),
            18 => format!("{:+012}", $v),
            19 => format!("{:+12}", $v),
            20 => format!("{:012.3}", $v),
            21 => format!("{:<012.2}", $v),
            22 => format!("{:*>12}", $v),
            23 => format!("{:*<12}", $v),
            24 => format!("{:*^13}", $v),
            25 => format!("{:0>12}", $v),
            26 => format!("{:0<12}", $v),
            27 => format!("{:#}", $v),
            28 => format!("{:#.2}", $v),
            29 => format!("{:#?}", $v),
            30 => format!("{:12.4e}", $v),
            31 => format!("{:+012.2e}", $v),
            32 => format!("{:.50}", $v),
            33 => format!("{:.100e}", $v),
            34 => format!("{:é>10}", $v),
            35 => format!("{:5}", $v),
            36 => format!("{:.500}", $v),
            37 => format!("{:^012}", $v),
            38 => format!("{:+}", $v),
            39 => format!("{:+?}", $v),
            40 => format!("{:+e}", $v),
            41 => format!("{:012e}", $v),
            42 => format!("{:12.0}", $v),
            43 => format!("{:^+12}", $v),
            _ => unreachable!(),
        }
    };
}

fn main() {
    let mode = std::env::args().nth(1).expect("mode argument");
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let stdout = std::io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    let mut line = String::new();
    loop {
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        // Only the newline is framing; the rest of the line is the case,
        // including any spaces (parse cases need them intact).
        if line.ends_with('\n') {
            line.pop();
        }
        match mode.as_str() {
            "fmt64" => {
                let v = f64::from_bits(u64::from_str_radix(&line, 16).unwrap());
                writeln!(out, "{}\t{:?}\t{:e}\t{:E}", v, v, v, v).unwrap();
            }
            "fmt32" => {
                let v = f32::from_bits(u32::from_str_radix(&line, 16).unwrap());
                writeln!(out, "{}\t{:?}\t{:e}\t{:E}", v, v, v, v).unwrap();
            }
            "spec64" | "spec32" => {
                let (bits, idx) = line.split_once(' ').unwrap();
                let idx: u32 = idx.parse().unwrap();
                if mode == "spec64" {
                    let v = f64::from_bits(u64::from_str_radix(bits, 16).unwrap());
                    writeln!(out, "{}", spec_format!(v, idx)).unwrap();
                } else {
                    let v = f32::from_bits(u32::from_str_radix(bits, 16).unwrap());
                    writeln!(out, "{}", spec_format!(v, idx)).unwrap();
                }
            }
            "parse64" => match line.parse::<f64>() {
                Ok(v) => writeln!(out, "0 {:016x}", v.to_bits()).unwrap(),
                Err(e) => writeln!(out, "{} {:016x}", if format!("{:?}", e).contains("Empty") { 1 } else { 2 }, 0u64).unwrap(),
            },
            "parse32" => match line.parse::<f32>() {
                Ok(v) => writeln!(out, "0 {:08x}", v.to_bits()).unwrap(),
                Err(e) => writeln!(out, "{} {:08x}", if format!("{:?}", e).contains("Empty") { 1 } else { 2 }, 0u32).unwrap(),
            },
            "cast64" => {
                let v = f64::from_bits(u64::from_str_radix(&line, 16).unwrap());
                writeln!(out, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:08x}", v as i8, v as i16, v as i32, v as i64, v as u8, v as u16, v as u32, v as u64, (v as f32).to_bits()).unwrap();
            }
            "cast32" => {
                let v = f32::from_bits(u32::from_str_radix(&line, 16).unwrap());
                writeln!(out, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:016x}", v as i8, v as i16, v as i32, v as i64, v as u8, v as u16, v as u32, v as u64, (v as f64).to_bits()).unwrap();
            }
            "cast128" => {
                let v = f64::from_bits(u64::from_str_radix(&line, 16).unwrap());
                writeln!(out, "{} {}", v as i128, v as u128).unwrap();
            }
            "cast128f" => {
                let v = f32::from_bits(u32::from_str_radix(&line, 16).unwrap());
                writeln!(out, "{} {}", v as i128, v as u128).unwrap();
            }
            "intcast" => {
                let (los, his) = line.split_once(' ').unwrap();
                let lo = u64::from_str_radix(los, 16).unwrap();
                let hi = u64::from_str_radix(his, 16).unwrap();
                let i = lo as i64;
                let u128v = ((hi as u128) << 64) | lo as u128;
                let i128v = u128v as i128;
                writeln!(
                    out,
                    "{:016x}\t{:016x}\t{:08x}\t{:08x}\t{:016x}\t{:016x}\t{:08x}\t{:08x}",
                    (i as f64).to_bits(),
                    (lo as f64).to_bits(),
                    (i as f32).to_bits(),
                    (lo as f32).to_bits(),
                    (i128v as f64).to_bits(),
                    (u128v as f64).to_bits(),
                    (i128v as f32).to_bits(),
                    (u128v as f32).to_bits()
                )
                .unwrap();
            }
            other => {
                eprintln!("bad mode {other}");
                std::process::exit(2);
            }
        }
    }
}

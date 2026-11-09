// Adapters over closures that capture the same borrow keep each closure's
// captures apart (ts-rust parsed_command_line.rs get_output_file_names).
pub trait Host { fn sens(&self) -> bool; }
pub struct O { decl: bool }
pub struct C { names: Vec<String>, o: O }
impl Host for C { fn sens(&self) -> bool { true } }
fn js(f: &str, _o: &O, h: &dyn Host) -> String { if h.sens() { format!("{f}.js") } else { String::new() } }
fn dts(f: &str, _o: &O, h: &dyn Host) -> String { if h.sens() { format!("{f}.d.ts") } else { String::new() } }
fn map(f: &str, _o: &O) -> String { format!("{f}.map") }
impl C {
    fn mapper(&self, f: &str) -> Option<u8> { if f.is_empty() { Some(1) } else { None } }
    pub fn outs(&self) -> impl Iterator<Item = String> + '_ {
        let opts = &self.o;
        self.names.iter().filter(|n| !n.is_empty()).flat_map(move |file_name| {
            let is_json = file_name.ends_with(".json");
            let js = std::iter::once_with(move || js(file_name, opts, self))
                .filter(|j| !j.is_empty())
                .flat_map(move |j| {
                    let sm = if is_json { String::new() } else { map(&j, opts) };
                    std::iter::once(j).chain((!sm.is_empty()).then_some(sm))
                });
            let d = std::iter::once_with(move || { if is_json || !opts.decl { return String::new(); } dts(file_name, opts, self) })
                .filter(|d| !d.is_empty())
                .flat_map(move |d| {
                    let dm = (self.mapper(file_name).is_none() && opts.decl).then(|| format!("{d}.map"));
                    std::iter::once(d).chain(dm)
                });
            js.chain(d)
        })
    }
}
fn main() { let c = C { names: vec!["a".into(), "b.json".into()], o: O { decl: true } }; for o in c.outs() { println!("{o}"); } }

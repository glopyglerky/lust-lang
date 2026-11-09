// expect: L0115
// rustc E0004: upstream `Resource` is `#[non_exhaustive]`.
use rustix::process::Resource;

fn n(r: Resource) -> u32 {
    match r {
        Resource::Cpu => 0, Resource::Fsize => 1, Resource::Data => 2, Resource::Stack => 3,
        Resource::Core => 4, Resource::Rss => 5, Resource::Nproc => 6, Resource::Nofile => 7,
        Resource::Memlock => 8, Resource::As => 9, Resource::Locks => 10, Resource::Sigpending => 11,
        Resource::Msgqueue => 12, Resource::Nice => 13, Resource::Rtprio => 14, Resource::Rttime => 15,
    }
}

fn main() {
    println!("{}", n(Resource::Nofile));
}

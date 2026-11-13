// The `self: &Rc<Self>` method shape from ts-rust
// frontend/packagejson.rs (`with_package_directory`).
use std::rc::Rc;

#[derive(Debug)]
struct InfoCacheEntry {
    package_directory: String,
    directory_exists: bool,
    contents: Option<Rc<String>>,
}

impl InfoCacheEntry {
    #[must_use]
    pub fn with_package_directory(self: &Rc<Self>, package_directory: &str) -> Rc<InfoCacheEntry> {
        if self.package_directory == package_directory {
            return self.clone();
        }
        Rc::new(InfoCacheEntry {
            package_directory: package_directory.to_string(),
            directory_exists: self.directory_exists,
            contents: self.contents.clone(),
        })
    }
}

fn main() {
    let e = Rc::new(InfoCacheEntry {
        package_directory: "a".to_string(),
        directory_exists: true,
        contents: Some(Rc::new("x".to_string())),
    });
    let same = e.with_package_directory("a");
    let other = e.with_package_directory("b");
    println!("{} {:?} {}", same.package_directory, other, Rc::strong_count(&e));
}

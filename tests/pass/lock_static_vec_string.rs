use std::sync::Mutex;
static V: Mutex<Vec<String>> = Mutex::new(Vec::new());
static S: Mutex<String> = Mutex::new(String::new());
fn main() { V.lock().unwrap().push(String::from("a")); S.lock().unwrap().push_str("xy"); println!("{:?} {}", V.lock().unwrap(), S.lock().unwrap()); }

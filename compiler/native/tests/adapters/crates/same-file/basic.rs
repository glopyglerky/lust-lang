// same-file 1.0.6: is_same_file over files and directories
// the program makes, with error forms. Identity values are
// compared, never printed raw.
use same_file::is_same_file;

fn main() {
    // Per process: the oracle and lust runs, or two gates on one box, must not share it.
    let dir = std::env::temp_dir().join(format!("lust_same_file_basic_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    std::fs::write(&a, "same contents").unwrap();
    std::fs::write(&b, "same contents").unwrap();

    // The ts-rust compile.rs form.
    println!("{}", is_same_file(&a, &a).unwrap_or(false));
    println!("{}", is_same_file(&a, &b).unwrap_or(false));
    println!("{}", is_same_file(&a, dir.join("./a.txt")).unwrap());
    println!("{}", is_same_file(&a, dir.join("sub/../a.txt")).unwrap());
    println!("{}", is_same_file(&a, a.to_str().unwrap()).unwrap());
    println!("{}", is_same_file(&dir, &dir).unwrap());
    println!("{}", is_same_file(&dir, &a).unwrap());
    println!("{}", is_same_file(dir.join("sub"), &dir).unwrap());
    // Errors: the kind and text are std's io errors.
    let missing = dir.join("nope.txt");
    let e1 = is_same_file(&missing, &a).unwrap_err();
    println!("{:?} {}", e1.kind(), e1);
    let e2 = is_same_file(&a, &missing).unwrap_err();
    println!("{:?} {}", e2.kind(), e2);
    let e3 = is_same_file(&missing, &missing).unwrap_err();
    println!("{:?}", e3.kind());
    println!("{}", is_same_file(&missing, &a).unwrap_or(false));
    let _ = std::fs::remove_dir_all(&dir);
}

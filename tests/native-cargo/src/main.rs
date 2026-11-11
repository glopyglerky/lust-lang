mod value;

fn main() {
    println!(
        "answer={} included={} env={} cache_env={} module={} nested={} file={} debug={} feature={}",
        value::answer(),
        include_str /* external-file */ !("../data.txt").trim(),
        option_env /* external-env */ !("LUSTR_NATIVE_INPUT").unwrap_or("unset"),
        option_env!("LUST_CACHE_DIR").unwrap_or("unset"),
        module_path!(),
        value::path(),
        file!(),
        cfg!(debug_assertions),
        cfg!(feature = "special"),
    );
}

macro_rules! textual_bytes {
    () => {
        b"hi"
    };
}

macro_rules! textual_raw {
    () => {
        br"raw"
    };
}

macro_rules! textual_byte {
    () => {
        b'k'
    };
}

#[macro_export]
macro_rules! path_bytes {
    () => {
        b"pa"
    };
}

#[macro_export]
macro_rules! path_byte {
    () => {
        b'p'
    };
}

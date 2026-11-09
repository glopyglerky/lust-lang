//! Regression for libc's variadic declaration sharing a symbol with std's
//! owned-file-descriptor cleanup. Keep the C signature's two fixed arguments.

use std::ffi::c_int;
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;

unsafe extern "C" {
    fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
}

const F_GETFD: c_int = 1;
const F_SETFD: c_int = 2;
const FD_CLOEXEC: c_int = 1;

fn check_file_descriptor_lifecycle() {
    let mut file = File::open("/dev/null").unwrap();
    let fd = file.as_raw_fd();
    // SAFETY: fd is owned by the live File. F_GETFD has no optional argument;
    // F_SETFD takes a promoted C int. No raw call closes the owned descriptor.
    unsafe {
        let original = fcntl(fd, F_GETFD);
        assert_ne!(original, -1);
        let toggled = original ^ FD_CLOEXEC;
        assert_eq!(fcntl(fd, F_SETFD, toggled), 0);
        assert_eq!(fcntl(fd, F_GETFD), toggled);
        assert_eq!(fcntl(fd, F_SETFD, original), 0);
        assert_eq!(fcntl(fd, F_GETFD), original);
    }
    let mut duplicate = file.try_clone().unwrap();
    assert_ne!(duplicate.as_raw_fd(), fd);
    assert_eq!(file.read(&mut [0_u8; 1]).unwrap(), 0);
    assert_eq!(duplicate.read(&mut [0_u8; 1]).unwrap(), 0);
    // std may instantiate debug_assert_fd_is_open here, including its own
    // fcntl(fd, F_GETFD) declaration. Both imports must agree on the fixed
    // parameter prefix without changing debug assertions or codegen units.
    drop(duplicate);
    drop(file);
}

fn main() {
    check_file_descriptor_lifecycle();
    println!("fcntl variadic passed: getfd=2-args setfd=3-args std-owned-fd-drops=2");
}

#[cfg(test)]
mod tests {
    #[test]
    fn variadic_fcntl_preserves_owned_file_lifecycle() {
        super::check_file_descriptor_lifecycle();
    }
}

//! The stack size of a thread that runs Go code: the work threads of the
//! binaries, the parse, bind, checker, emit, search and goroutine threads,
//! the `tsc -b` config and build info threads, the file watcher thread and
//! the LSP read thread.
//!
//! Go: a goroutine stack starts small and grows up to the maximum stack
//! size (runtime `maxstacksize`, 1e9 bytes on 64-bit systems). Past it, Go
//! stops with "goroutine stack exceeds 1000000000-byte limit".
//! PORT: a Rust thread stack does not grow, so each thread reserves the Go
//! maximum when it starts. The reservation is only address space, but an
//! address space or data limit (`ulimit -v`, `ulimit -d`) counts all of it:
//! ten 1 GiB stacks need 10 GiB, where Go checks Query core in less than
//! 2 GiB. Under such a limit, a stack gets 1/64 of the limit. A deep input
//! can then overflow the stack sooner than in Go, and the port stops with
//! the Rust stack overflow message.

use std::sync::OnceLock;

/// The stack size with no limit: the Go maximum (1e9 bytes), rounded up.
const MAX_STACK_SIZE: usize = 1 << 30;

/// The smallest stack under a limit: the Linux default for a main thread.
const MIN_STACK_SIZE: usize = 8 << 20;

/// Under a limit, each stack gets 1/64 of it. The share does not count the
/// threads: 32 stacks take half of the limit and 64 take all of it. Query
/// core, where Go exits 0 in each case: `--checkers 16` and `--checkers 32`
/// give Go's output at `ulimit -v 1G` and `2G`. `--checkers 48` does at 2G,
/// but exits 70 ("cannot start a checker thread") at 1G. `--checkers 64`
/// exits 70 at both.
const LIMIT_SHARE: u64 = 64;


/// The stack size under the address space limit `limit` (`None`: no limit).
fn stack_size_for(limit: Option<u64>) -> usize {
    limit.map_or(MAX_STACK_SIZE, |limit| {
        usize::try_from(limit / LIMIT_SHARE)
            .unwrap_or(MAX_STACK_SIZE)
            .clamp(MIN_STACK_SIZE, MAX_STACK_SIZE)
    })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_size_follows_the_limit() {
        const GIB: u64 = 1 << 30;
        assert_eq!(stack_size_for(None), 1 << 30);
        assert_eq!(stack_size_for(Some(128 * GIB)), 1 << 30);
        assert_eq!(stack_size_for(Some(8 * GIB)), 128 << 20);
        assert_eq!(stack_size_for(Some(2 * GIB)), 32 << 20);
        assert_eq!(stack_size_for(Some(GIB / 4)), 8 << 20);
    }
}

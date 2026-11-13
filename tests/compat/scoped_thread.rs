// K1 compatibility fixture: a scoped thread over a borrowed slice.
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/program.rs parallel bind: a
// std::thread::scope where each worker borrows the files slice and
// the order vec (no 'static), clones an mpsc sender, and the loading
// thread collects (index, result) pairs. ts-rust spawns through a
// GoThread wrapper for stack size; the scope and borrows are the
// same shape here with scope.spawn.
//
// Output is sorted by index: worker finish order is not observable.

use std::sync::Mutex;

/// Bind one file: count its bytes and its first word length.
fn bind_file(text: &str) -> (usize, usize) {
    let first_word = text.split_whitespace().next().unwrap_or("").len();
    (text.len(), first_word)
}

fn main() {
    let files: Vec<&str> = vec![
        "const a = 1;",
        "export function longerName() { return 2; }",
        "x",
        "// just a comment\nconst b = 2;",
        "",
        "interface Foo { bar: string }",
        "type T = A | B | C;",
        "namespace N { export const v = 3; }",
    ];
    // Longest first, as in the real queue.
    let mut order: Vec<usize> = (0..files.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(files[i].len()));

    let shared_total = Mutex::new(0usize);
    let (sender, receiver) = std::sync::mpsc::channel::<(usize, (usize, usize))>();

    let threads = 3;
    std::thread::scope(|scope| {
        for _ in 0..threads {
            // Borrowed slice and order, cloned sender: the scoped
            // spawn may outlive neither.
            let (files, order, total, sender) =
                (&files, &order, &shared_total, sender.clone());
            scope.spawn(move || {
                let mut local = 0usize;
                for &i in order.iter() {
                    let result = bind_file(files[i]);
                    local += result.0;
                    let _ = sender.send((i, result));
                }
                *total.lock().unwrap() += local;
            });
        }
    });
    drop(sender);

    let mut results: Vec<(usize, (usize, usize))> = receiver.iter().collect();
    results.sort_by_key(|&(i, _)| i);
    // Three workers each bound every file (the queue stand-in does
    // not partition work); keep the first result per index.
    let mut seen = std::collections::HashSet::new();
    for (i, (bytes, word)) in &results {
        if seen.insert(*i) {
            println!("file {i}: bytes={bytes} first_word={word}");
        }
    }
    println!("unique files: {}", seen.len());
    println!("total bytes over {threads} workers: {}", shared_total.lock().unwrap());
}

// Collecting lust's builtin std types into a Vec from an iterator of unknown length needs
// their layout (RawVec's first capacity), and size_of needs it too. tf-build: HashMap,
// HashSet, RandomState, DefaultHasher, JoinHandle, Thread, ThreadId, mpsc ends, Condvar,
// Barrier, Range, RangeInclusive, Chars, CharIndices, vec::IntoIter and ExitCode passed
// check and failed the build ("iterator expansion: element layout unknown").
use std::collections::hash_map::{DefaultHasher, RandomState};
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;

macro_rules! sizes {
    ($($t:ty),* $(,)?) => {
        $(println!("{} {} {} {}", std::mem::size_of::<$t>(), std::mem::align_of::<$t>(), std::mem::size_of::<Option<$t>>(), std::mem::size_of::<Option<Option<$t>>>());)*
    };
}

fn odd(x: &u32) -> bool {
    x % 2 == 1
}

fn main() {
    sizes!(
        HashMap<u32, u32>,
        HashMap<u8, u8, BuildHasherDefault<DefaultHasher>>,
        HashSet<String>,
        RandomState,
        DefaultHasher,
        std::thread::JoinHandle<u8>,
        std::thread::Thread,
        std::thread::ThreadId,
        std::sync::mpsc::Sender<u8>,
        std::sync::mpsc::Receiver<u8>,
        std::sync::Condvar,
        std::sync::Barrier,
        std::ops::Range<u8>,
        std::ops::RangeInclusive<u32>,
        std::str::Chars<'static>,
        std::str::CharIndices<'static>,
        std::vec::IntoIter<String>,
        std::process::ExitCode,
    );
    let maps: Vec<HashMap<u32, u32>> = (0..7u32).filter(odd).map(|_| HashMap::new()).collect();
    let sets: Vec<HashSet<u8>> = (0..3u32).filter(odd).map(|_| HashSet::new()).collect();
    let ranges: Vec<std::ops::Range<u32>> = (0..9u32).filter(odd).map(|i| i..i + 1).collect();
    let chars: Vec<std::str::Chars<'static>> = (0..5u32).filter(odd).map(|_| "ab".chars()).collect();
    let codes: Vec<std::process::ExitCode> = (0..5u32).filter(odd).map(|_| std::process::ExitCode::SUCCESS).collect();
    let handles: Vec<std::thread::JoinHandle<u32>> = (0..5u32).filter(odd).map(|i| std::thread::spawn(move || i * 2)).collect();
    let (tx, rx) = std::sync::mpsc::channel::<u8>();
    let senders: Vec<std::sync::mpsc::Sender<u8>> = (0..5u32).filter(odd).map(|_| tx.clone()).collect();
    println!("{} {} {} {} {}", maps.capacity(), sets.capacity(), ranges.capacity(), chars.capacity(), codes.capacity());
    let sum: u32 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    for s in &senders {
        s.send(1).unwrap();
    }
    drop(senders);
    drop(tx);
    println!("{} {} {}", sum, rx.iter().count(), ranges.iter().map(|r| r.start).sum::<u32>());
}

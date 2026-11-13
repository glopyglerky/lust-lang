// Adapters and consumers over iterators reached through their own
// `next` (W-STD-B): user iterator types, `I: Iterator` parameters, and
// `<I as IntoIterator>::IntoIter`; the trait's default size_hint
// decides collect's capacity.
struct Countdown {
    n: u32,
}

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n == 0 {
            return None;
        }
        self.n -= 1;
        Some(self.n)
    }
}

struct Words<'a> {
    s: &'a [&'a str],
    i: usize,
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let w = self.s.get(self.i)?;
        self.i += 1;
        Some(*w)
    }
}

fn evens<I: Iterator<Item = u32>>(it: I) -> Vec<u32> {
    it.filter(|x| x % 2 == 0).collect()
}

fn total<I: IntoIterator<Item = u32>>(it: I) -> u32 {
    it.into_iter().map(|x| x * 10).sum()
}

fn longest(it: impl Iterator<Item = usize>) -> Option<usize> {
    it.max()
}

fn main() {
    let v: Vec<u32> = Countdown { n: 5 }.map(|x| x + 1).collect();
    println!("{v:?} {}", v.capacity());
    let w: Vec<u32> = Countdown { n: 6 }.filter(|x| x % 3 == 0).collect();
    println!("{w:?} {}", w.capacity());
    println!("{:?}", evens(Countdown { n: 7 }));
    println!("{:?}", evens(vec![1u32, 2, 4].into_iter()));
    println!("{} {}", total(vec![3u32, 4]), total(vec![1u32, 2]));
    let src = ["alpha", "be", "gamma"];
    let lens: Vec<usize> = Words { s: &src[..], i: 0 }.map(|w| w.len()).collect();
    println!("{lens:?} {:?}", longest(lens.iter().copied()));
    let joined: Vec<(usize, &str)> = Words { s: &src[..], i: 1 }.enumerate().collect();
    println!("{joined:?}");
    let mut pk = Countdown { n: 3 }.peekable();
    let first = match pk.peek() {
        Some(x) => Some(*x),
        None => None,
    };
    let a = pk.next();
    println!("{first:?} {a:?} {:?}", pk.next());
    let both: Vec<u32> = Countdown { n: 2 }.chain(Countdown { n: 2 }).collect();
    let z: Vec<(u32, &str)> = Countdown { n: 9 }.zip(Words { s: &src[..], i: 0 }).collect();
    println!("{both:?} {z:?}");
    println!("{} {:?} {:?}", Countdown { n: 4 }.count(), Countdown { n: 4 }.last(), Countdown { n: 4 }.nth(1));
    println!("{} {}", Countdown { n: 4 }.any(|x| x == 2), Countdown { n: 4 }.all(|x| x < 4));
    println!("{:?} {:?}", Countdown { n: 4 }.find(|x| *x < 2), Countdown { n: 4 }.position(|x| x == 1));
    println!("{}", Countdown { n: 5 }.fold(0, |a, x| a * 2 + x));
    Countdown { n: 2 }.for_each(|x| println!("each {x}"));
    let s: Vec<u32> = Countdown { n: 10 }.skip(2).step_by(3).take(2).collect();
    println!("{s:?}");
    let m: Vec<String> = Words { s: &src[..], i: 0 }.take_while(|w| w.len() > 2).map(|w| w.to_string()).collect();
    println!("{m:?}");
}

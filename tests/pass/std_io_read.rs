// std::io Read/BufRead/BufReader/BufWriter/stdin (W-SHIM phase 2):
// slice readers, buffered readers and writers, lines, read_line,
// read_to_end/read_to_string (incl. error and invalid UTF-8 rules),
// by_ref, a user Read impl, and Debug forms.
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

struct Chunked {
    data: Vec<u8>,
    pos: usize,
    per: usize,
}

impl Read for Chunked {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.per.min(self.data.len() - self.pos).min(buf.len());
        for i in 0..n {
            buf[i] = self.data[self.pos + i];
        }
        self.pos += n;
        Ok(n)
    }
}

fn drain<R: Read>(mut r: R) -> Vec<u8> {
    let mut v = Vec::new();
    let _ = r.read_to_end(&mut v);
    v
}

fn main() {
    // &[u8] as a reader, read + read_exact + by_ref.
    let mut s: &[u8] = b"hello world";
    let mut b = [0u8; 5];
    println!("{:?}", s.read(&mut b));
    println!("{:?} {:?}", b, s);
    println!("{:?}", s.by_ref().read_exact(&mut b));
    println!("{:?} {:?}", b, s);
    let mut rest = Vec::new();
    println!("{:?} {:?} {:?}", s.read_to_end(&mut rest), rest, s);
    let mut s2: &[u8] = b"abc";
    let mut b2 = [9u8; 6];
    match s2.read_exact(&mut b2) {
        Ok(()) => println!("unexpected ok"),
        Err(e) => println!("{} {:?} {:?} {:?}", e, e.kind(), b2, s2),
    }
    // read_to_string: valid, invalid (buffer restored), pre-filled.
    let mut text = String::from(">");
    let mut rd: &[u8] = b"ab\ncd";
    println!("{:?} {:?}", rd.read_to_string(&mut text), text);
    let mut text = String::from("keep");
    let mut rd: &[u8] = b"a\xffb";
    match rd.read_to_string(&mut text) {
        Ok(n) => println!("unexpected {n}"),
        Err(e) => println!("{} {:?} {:?}", e, e.kind(), text),
    }
    // A user reader through a generic fn and through BufReader.
    let c = Chunked { data: b"0123456789".to_vec(), pos: 0, per: 3 };
    println!("{:?}", drain(c));
    let c = Chunked { data: b"line1\nline2\nline3".to_vec(), pos: 0, per: 4 };
    let br = BufReader::with_capacity(8, c);
    for line in br.lines() {
        println!("{:?}", line);
    }
    // BufReader: buffer/capacity/fill_buf/consume/get_ref/into_inner.
    let data1: &[u8] = b"abcdefghij";
    let data2: &[u8] = b"one\ntwo\nthree";
    let data3: &[u8] = b"ok\nbad\xff\nlast\n";
    let data4: &[u8] = b"a\r\nb\n\nlast";
    let data5: &[u8] = b"hi";
    let mut br = BufReader::with_capacity(4, data1);
    println!("cap {} buf {:?}", br.capacity(), br.buffer());
    match br.fill_buf() {
        Ok(x) => println!("fill Ok({:?})", x.to_vec()),
        Err(e) => println!("fill Err({e})"),
    }
    br.consume(3);
    println!("buf {:?}", br.buffer());
    let mut one = [0u8; 2];
    println!("{:?} {:?}", br.read(&mut one), one);
    println!("{:?}", br.get_ref());
    println!("into {:?}", br.into_inner());
    // read_line appends and counts.
    let mut br = BufReader::new(data2);
    let mut line = String::from("#");
    println!("{:?} {:?}", br.read_line(&mut line), line);
    line.clear();
    println!("{:?} {:?}", br.read_line(&mut line), line);
    line.clear();
    println!("{:?} {:?}", br.read_line(&mut line), line);
    line.clear();
    println!("{:?} {:?}", br.read_line(&mut line), line);
    // read_line with invalid UTF-8: not appended, still consumed.
    let mut br = BufReader::new(data3);
    let mut line = String::new();
    println!("{:?} {:?}", br.read_line(&mut line), line);
    match br.read_line(&mut line) {
        Ok(n) => println!("unexpected {n}"),
        Err(e) => println!("{} {:?} {:?}", e, e.kind(), line),
    }
    println!("{:?} {:?}", br.read_line(&mut line), line);
    // lines(): CRLF, empty lines, no trailing newline.
    let mut ls: Vec<io::Result<String>> = Vec::new();
    for l in BufReader::new(data4).lines() {
        ls.push(l);
    }
    println!("{:?}", ls);
    // &[u8] is also BufRead.
    let mut sl: &[u8] = b"x\ny";
    match sl.fill_buf() {
        Ok(x) => println!("Ok({:?})", x.to_vec()),
        Err(e) => println!("Err({e})"),
    }
    sl.consume(2);
    let mut line = String::new();
    println!("{:?} {:?}", sl.read_line(&mut line), line);
    // BufWriter: buffering, flush writes through, buffer()/capacity().
    let mut bw = BufWriter::with_capacity(8, Vec::new());
    bw.write_all(b"abc").unwrap();
    println!("bw {:?} {:?} {}", bw.get_ref(), bw.buffer(), bw.capacity());
    bw.write_all(b"defghij").unwrap();
    println!("bw {:?} {:?}", bw.get_ref(), bw.buffer());
    bw.write_all(b"0123456789").unwrap();
    println!("bw {:?} {:?}", bw.get_ref(), bw.buffer());
    bw.flush().unwrap();
    println!("bw {:?} {:?}", bw.get_ref(), bw.buffer());
    // BufWriter flushes on drop.
    let mut sink: Vec<u8> = Vec::new();
    {
        let mut bw = BufWriter::new(&mut sink);
        bw.write_all(b"dropme").unwrap();
    }
    println!("dropped {:?}", sink);
    // Debug forms.
    println!("{:?}", BufReader::new(data5));
    println!("{:?}", BufWriter::new(Vec::<u8>::new()));
    println!("{:?}", io::stdin());
    // stdin at EOF (difftest runs with empty stdin).
    let mut line = String::new();
    println!("stdin {:?}", io::stdin().read_line(&mut line));
    let mut count = 0;
    for _l in io::stdin().lock().lines() {
        count += 1;
    }
    println!("stdin lock {count}");
    let mut count = 0;
    for _l in io::stdin().lines() {
        count += 1;
    }
    println!("stdin lines {count}");
}

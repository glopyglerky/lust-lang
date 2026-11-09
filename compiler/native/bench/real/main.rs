// logscan: a realistic single-file log analyzer, written in the lust subset
// (which is plain Rust: rustc and lust must both build it, with the same output).
//
// It generates a deterministic synthetic service log, parses it back, and prints
// an operations report: per-service latency percentiles, error bursts, hourly
// volume, slow requests, and a filtered incident view. It exists as a benchmark
// corpus of varied, hand-written code (not cloned modules): see bench/run.py.
//
// The subset has no HashMap, no floats, and no string slicing, so maps are
// Vecs of structs, percentages are integer permille, and text work goes
// through Vec<char>.

// ---------- Deterministic random numbers (xorshift64*) ----------

#[derive(Debug, Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Rng {
        // Any odd nonzero seed works; stir the given seed so small seeds differ.
        let s = seed ^ 0x9E3779B97F4A7C15;
        Rng { state: if s == 0 { 0x2545F4914F6CDD1D } else { s } }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x = x ^ (x >> 12);
        x = x ^ (x << 25);
        x = x ^ (x >> 27);
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }

    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            lo
        } else {
            lo + self.below((hi - lo) as u64) as i64
        }
    }

    fn chance(&mut self, permille: u64) -> bool {
        self.below(1000) < permille
    }

    fn pick(&mut self, items: &Vec<String>) -> String {
        if items.is_empty() {
            String::new()
        } else {
            items[self.below(items.len() as u64) as usize].clone()
        }
    }
}

// ---------- Small text helpers (the subset has no str slicing) ----------

fn to_chars(s: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in s.chars() {
        out.push(c);
    }
    out
}

fn chars_to_string(chars: &Vec<char>, start: usize, end: usize) -> String {
    let mut out = String::new();
    let mut i = start;
    while i < end && i < chars.len() {
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn trim(s: &str) -> String {
    let chars = to_chars(s);
    let mut lo = 0;
    let mut hi = chars.len();
    while lo < hi && chars[lo].is_ascii_whitespace() {
        lo += 1;
    }
    while hi > lo && chars[hi - 1].is_ascii_whitespace() {
        hi -= 1;
    }
    chars_to_string(&chars, lo, hi)
}

/// Splits on runs of whitespace, like shell words without quoting.
fn split_ws(s: &str) -> Vec<String> {
    let chars = to_chars(s);
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_ascii_whitespace() {
            i += 1;
        }
        if i > start {
            out.push(chars_to_string(&chars, start, i));
        }
    }
    out
}

/// Splits on one separator character, keeping empty fields.
fn split_on(s: &str, sep: char) -> Vec<String> {
    let chars = to_chars(s);
    let mut out: Vec<String> = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == sep {
            out.push(chars_to_string(&chars, start, i));
            start = i + 1;
        }
        i += 1;
    }
    out.push(chars_to_string(&chars, start, chars.len()));
    out
}

fn starts_with(s: &str, prefix: &str) -> bool {
    let a = to_chars(s);
    let b = to_chars(prefix);
    if b.len() > a.len() {
        return false;
    }
    let mut i = 0;
    while i < b.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

fn contains(s: &str, needle: &str) -> bool {
    let a = to_chars(s);
    let b = to_chars(needle);
    if b.is_empty() {
        return true;
    }
    if b.len() > a.len() {
        return false;
    }
    let mut i = 0;
    while i + b.len() <= a.len() {
        let mut j = 0;
        let mut ok = true;
        while j < b.len() {
            if a[i + j] != b[j] {
                ok = false;
                break;
            }
            j += 1;
        }
        if ok {
            return true;
        }
        i += 1;
    }
    false
}

fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Parses a non-negative decimal integer. None on any bad character.
fn parse_u64(s: &str) -> Option<u64> {
    let chars = to_chars(s);
    if chars.is_empty() {
        return None;
    }
    let mut n: u64 = 0;
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            return None;
        }
        n = n * 10 + chars[i].to_digit(10).unwrap() as u64;
        i += 1;
    }
    Some(n)
}

/// Parses a signed decimal integer (one leading '-' is allowed).
fn parse_i64(s: &str) -> Option<i64> {
    let t = trim(s);
    if starts_with(&t, "-") {
        let chars = to_chars(&t);
        let rest = chars_to_string(&chars, 1, chars.len());
        match parse_u64(&rest) {
            Some(n) => Some(-(n as i64)),
            None => None,
        }
    } else {
        match parse_u64(&t) {
            Some(n) => Some(n as i64),
            None => None,
        }
    }
}

fn pad_right(s: &str, width: usize) -> String {
    let mut out = s.to_string();
    while out.len() < width {
        out.push(' ');
    }
    out
}

fn pad_left(s: &str, width: usize) -> String {
    let mut out = String::new();
    while out.len() + s.len() < width {
        out.push(' ');
    }
    out.push_str(s);
    out
}

fn pad2(n: i64) -> String {
    if n >= 0 && n < 10 {
        let mut s = String::from("0");
        s.push_str(&format!("{}", n));
        s
    } else {
        format!("{}", n)
    }
}

/// Formats permille (123 = 12.3%) with one decimal digit, integer math only.
fn fmt_permille(p: i64) -> String {
    let whole = p / 10;
    let frac = (p % 10).abs();
    format!("{}.{}", whole, frac)
}

/// Formats minutes-since-midnight as HH:MM.
fn fmt_clock(minutes: i64) -> String {
    let h = (minutes / 60) % 24;
    let m = minutes % 60;
    let mut s = pad2(h);
    s.push(':');
    s.push_str(&pad2(m));
    s
}

fn repeat_char(c: char, n: i64) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        out.push(c);
        i += 1;
    }
    out
}

// ---------- Log model ----------

#[derive(Debug, Clone, Copy, PartialEq)]
enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

fn level_rank(l: Level) -> i64 {
    match l {
        Level::Trace => 0,
        Level::Debug => 1,
        Level::Info => 2,
        Level::Warn => 3,
        Level::Error => 4,
        Level::Fatal => 5,
    }
}

fn level_name(l: Level) -> String {
    let n = match l {
        Level::Trace => "TRACE",
        Level::Debug => "DEBUG",
        Level::Info => "INFO",
        Level::Warn => "WARN",
        Level::Error => "ERROR",
        Level::Fatal => "FATAL",
    };
    n.to_string()
}

fn parse_level(s: &str) -> Option<Level> {
    let t = to_lower(&trim(s));
    if t == "trace" {
        Some(Level::Trace)
    } else if t == "debug" {
        Some(Level::Debug)
    } else if t == "info" {
        Some(Level::Info)
    } else if t == "warn" || t == "warning" {
        Some(Level::Warn)
    } else if t == "error" || t == "err" {
        Some(Level::Error)
    } else if t == "fatal" || t == "panic" {
        Some(Level::Fatal)
    } else {
        None
    }
}

fn level_is_bad(l: Level) -> bool {
    level_rank(l) >= level_rank(Level::Error)
}

/// One parsed log record. `minute` is minutes since midnight of the log day.
#[derive(Debug, Clone)]
struct LogLine {
    minute: i64,
    level: Level,
    service: String,
    message: String,
    latency_ms: i64,
    status: i64,
    request_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    EmptyLine,
    BadTimestamp(String),
    BadLevel(String),
    MissingField(String),
    BadNumber { field: String, text: String },
}

fn describe_parse_error(e: &ParseError) -> String {
    match e {
        ParseError::EmptyLine => "empty line".to_string(),
        ParseError::BadTimestamp(t) => format!("bad timestamp {}", t),
        ParseError::BadLevel(t) => format!("bad level {}", t),
        ParseError::MissingField(f) => format!("missing field {}", f),
        ParseError::BadNumber { field, text } => format!("bad number in {}: {}", field, text),
    }
}

/// Parses "2026-01-15T13:42:07" into minutes since midnight. Seconds are checked, then dropped.
fn parse_timestamp(s: &str) -> Option<i64> {
    let halves = split_on(s, 'T');
    if halves.len() != 2 {
        return None;
    }
    let date = split_on(&halves[0], '-');
    if date.len() != 3 {
        return None;
    }
    let year_ok = parse_u64(&date[0]);
    let month_ok = parse_u64(&date[1]);
    let day_ok = parse_u64(&date[2]);
    if year_ok.is_none() || month_ok.is_none() || day_ok.is_none() {
        return None;
    }
    let time = split_on(&halves[1], ':');
    if time.len() != 3 {
        return None;
    }
    let h = parse_u64(&time[0]);
    let m = parse_u64(&time[1]);
    let sec = parse_u64(&time[2]);
    if h.is_none() || m.is_none() || sec.is_none() {
        return None;
    }
    let h = h.unwrap() as i64;
    let m = m.unwrap() as i64;
    if h > 23 || m > 59 || sec.unwrap() > 59 {
        return None;
    }
    Some(h * 60 + m)
}

/// Finds `key=value` in a word list. Value runs to the end of the word.
fn field_value(words: &Vec<String>, key: &str) -> Option<String> {
    let mut prefix = key.to_string();
    prefix.push('=');
    let mut i = 0;
    while i < words.len() {
        if starts_with(&words[i], &prefix) {
            let chars = to_chars(&words[i]);
            return Some(chars_to_string(&chars, prefix.len(), chars.len()));
        }
        i += 1;
    }
    None
}

/// Parses one log line:
///   2026-01-15T13:42:07 INFO api req=991 msg="served" ms=12 status=200
/// The message may contain spaces only inside msg="...". Returns ParseError
/// in `err` and None when the line is bad, Some(record) and no error when good.
/// (Two out-params by value: this subset has no Result type yet.)
#[derive(Debug, Clone)]
struct Parsed {
    line: Option<LogLine>,
    error: Option<ParseError>,
}

fn parse_line(raw: &str) -> Parsed {
    let text = trim(raw);
    if text.is_empty() {
        return Parsed { line: None, error: Some(ParseError::EmptyLine) };
    }
    let words = split_ws(&text);
    if words.len() < 3 {
        return Parsed { line: None, error: Some(ParseError::MissingField("header".to_string())) };
    }
    let minute = match parse_timestamp(&words[0]) {
        Some(m) => m,
        None => return Parsed { line: None, error: Some(ParseError::BadTimestamp(words[0].clone())) },
    };
    let level = match parse_level(&words[1]) {
        Some(l) => l,
        None => return Parsed { line: None, error: Some(ParseError::BadLevel(words[1].clone())) },
    };
    let service = words[2].clone();
    let req = match field_value(&words, "req") {
        Some(v) => match parse_u64(&v) {
            Some(n) => n,
            None => return Parsed { line: None, error: Some(ParseError::BadNumber { field: "req".to_string(), text: v }) },
        },
        None => 0,
    };
    let latency = match field_value(&words, "ms") {
        Some(v) => match parse_i64(&v) {
            Some(n) => n,
            None => return Parsed { line: None, error: Some(ParseError::BadNumber { field: "ms".to_string(), text: v }) },
        },
        None => -1,
    };
    let status = match field_value(&words, "status") {
        Some(v) => match parse_i64(&v) {
            Some(n) => n,
            None => return Parsed { line: None, error: Some(ParseError::BadNumber { field: "status".to_string(), text: v }) },
        },
        None => 0,
    };
    let message = extract_message(&text);
    Parsed {
        line: Some(LogLine { minute, level, service, message, latency_ms: latency, status, request_id: req }),
        error: None,
    }
}

/// Reads the msg="..." payload out of a raw line, spaces included.
fn extract_message(text: &str) -> String {
    let chars = to_chars(text);
    let tag = to_chars("msg=\"");
    let mut i = 0;
    while i + tag.len() <= chars.len() {
        let mut j = 0;
        let mut ok = true;
        while j < tag.len() {
            if chars[i + j] != tag[j] {
                ok = false;
                break;
            }
            j += 1;
        }
        if ok {
            let start = i + tag.len();
            let mut end = start;
            while end < chars.len() && chars[end] != '"' {
                end += 1;
            }
            return chars_to_string(&chars, start, end);
        }
        i += 1;
    }
    String::new()
}

#[derive(Debug)]
struct ParseReport {
    lines: Vec<LogLine>,
    rejected: i64,
    first_error: String,
}

fn parse_log(text_lines: &Vec<String>) -> ParseReport {
    let mut out: Vec<LogLine> = Vec::new();
    let mut rejected = 0;
    let mut first_error = String::new();
    let mut i = 0;
    while i < text_lines.len() {
        let p = parse_line(&text_lines[i]);
        let parsed_line = p.line.clone();
        if let Some(err) = p.error {
            if err != ParseError::EmptyLine {
                rejected += 1;
                if first_error.is_empty() {
                    first_error = describe_parse_error(&err);
                }
            }
        }
        if let Some(line) = parsed_line {
            out.push(line);
        }
        i += 1;
    }
    ParseReport { lines: out, rejected, first_error }
}

// ---------- Synthetic log generation ----------

#[derive(Debug, Clone)]
struct ServiceProfile {
    name: String,
    base_ms: i64,
    error_permille: u64,
    weight: i64,
}

fn service_profiles() -> Vec<ServiceProfile> {
    vec![
        ServiceProfile { name: "api".to_string(), base_ms: 14, error_permille: 18, weight: 34 },
        ServiceProfile { name: "checkout".to_string(), base_ms: 48, error_permille: 26, weight: 14 },
        ServiceProfile { name: "search".to_string(), base_ms: 31, error_permille: 12, weight: 18 },
        ServiceProfile { name: "auth".to_string(), base_ms: 9, error_permille: 8, weight: 12 },
        ServiceProfile { name: "billing".to_string(), base_ms: 63, error_permille: 30, weight: 8 },
        ServiceProfile { name: "worker".to_string(), base_ms: 120, error_permille: 44, weight: 9 },
        ServiceProfile { name: "gateway".to_string(), base_ms: 6, error_permille: 5, weight: 5 },
    ]
}

fn message_templates() -> Vec<String> {
    vec![
        "request served".to_string(),
        "cache hit".to_string(),
        "cache miss".to_string(),
        "db query ok".to_string(),
        "db query slow".to_string(),
        "upstream timeout".to_string(),
        "retry scheduled".to_string(),
        "job dequeued".to_string(),
        "job finished".to_string(),
        "token refreshed".to_string(),
        "rate limit near".to_string(),
        "connection reset by peer".to_string(),
        "invalid input rejected".to_string(),
        "payment authorized".to_string(),
        "payment declined".to_string(),
        "index rebuild step".to_string(),
        "health check ok".to_string(),
        "config reloaded".to_string(),
    ]
}

fn pick_service(rng: &mut Rng, profiles: &Vec<ServiceProfile>) -> usize {
    let mut total = 0;
    let mut i = 0;
    while i < profiles.len() {
        total += profiles[i].weight;
        i += 1;
    }
    let mut roll = rng.range(0, total);
    let mut j = 0;
    while j < profiles.len() {
        roll -= profiles[j].weight;
        if roll < 0 {
            return j;
        }
        j += 1;
    }
    profiles.len() - 1
}

/// An incident window: one service fails hard for a stretch of minutes.
#[derive(Debug, Clone)]
struct Incident {
    service: String,
    start_minute: i64,
    end_minute: i64,
    extra_error_permille: u64,
    slow_factor: i64,
}

fn planned_incidents() -> Vec<Incident> {
    vec![
        Incident { service: "checkout".to_string(), start_minute: 9 * 60 + 12, end_minute: 9 * 60 + 47, extra_error_permille: 520, slow_factor: 6 },
        Incident { service: "worker".to_string(), start_minute: 13 * 60 + 3, end_minute: 13 * 60 + 29, extra_error_permille: 640, slow_factor: 4 },
        Incident { service: "search".to_string(), start_minute: 16 * 60 + 41, end_minute: 16 * 60 + 58, extra_error_permille: 380, slow_factor: 9 },
    ]
}

fn incident_for(incidents: &Vec<Incident>, service: &str, minute: i64) -> Option<Incident> {
    let mut i = 0;
    while i < incidents.len() {
        if incidents[i].service == service && minute >= incidents[i].start_minute && minute <= incidents[i].end_minute {
            return Some(incidents[i].clone());
        }
        i += 1;
    }
    None
}

fn format_timestamp(minute: i64, second: i64) -> String {
    let mut s = String::from("2026-01-15T");
    s.push_str(&fmt_clock(minute));
    s.push(':');
    s.push_str(&pad2(second));
    s
}

/// One generated raw log line. Field order matches parse_line: ms/status come
/// before msg so whitespace splitting cannot eat them.
fn generate_line(rng: &mut Rng, profiles: &Vec<ServiceProfile>, incidents: &Vec<Incident>, templates: &Vec<String>, minute: i64, req: u64) -> String {
    let si = pick_service(rng, profiles);
    let svc = profiles[si].clone();
    let mut err_chance = svc.error_permille;
    let mut slow = 1;
    if let Some(inc) = incident_for(incidents, &svc.name, minute) {
        err_chance += inc.extra_error_permille;
        slow = inc.slow_factor;
    }
    let failed = rng.chance(err_chance);
    let mut level = Level::Info;
    let mut status = 200;
    if failed {
        if rng.chance(60) {
            level = Level::Fatal;
            status = 503;
        } else {
            level = Level::Error;
            status = 500;
        }
    } else if rng.chance(70) {
        level = Level::Warn;
        status = 429;
    } else if rng.chance(110) {
        level = Level::Debug;
    }
    let jitter = rng.range(0, svc.base_ms + 4);
    let spike = if rng.chance(25) { rng.range(200, 2400) } else { 0 };
    let ms = (svc.base_ms + jitter) * slow + spike;
    let msg = rng.pick(templates);
    let ts = format_timestamp(minute, rng.range(0, 60));
    format!("{} {} {} req={} ms={} status={} msg=\"{}\"", ts, level_name(level), svc.name, req, ms, status, msg)
}

/// Builds the full synthetic log text, one String per line, a few bad lines mixed in.
fn generate_log(seed: u64, count: i64) -> Vec<String> {
    let profiles = service_profiles();
    let incidents = planned_incidents();
    let templates = message_templates();
    let mut rng = Rng::new(seed);
    let mut out: Vec<String> = Vec::new();
    let mut i: i64 = 0;
    let mut minute: i64 = 8 * 60;
    while i < count {
        if rng.chance(140) {
            minute += 1;
        }
        if minute > 18 * 60 + 30 {
            minute = 8 * 60;
        }
        out.push(generate_line(&mut rng, &profiles, &incidents, &templates, minute, 1000 + i as u64));
        i += 1;
    }
    // A handful of damaged lines, as real logs have.
    out.push("2026-01-15T99:99:99 INFO api req=1 ms=3 status=200 msg=\"bad clock\"".to_string());
    out.push("2026-01-15T10:00:00 NOTICE api req=2 ms=3 status=200 msg=\"odd level\"".to_string());
    out.push("2026-01-15T10:00:01 INFO api req=3 ms=fast status=200 msg=\"bad ms\"".to_string());
    out.push("truncated line from a crashed writer".to_string());
    out.push("".to_string());
    out
}

// ---------- Statistics ----------

/// Insertion sort, in place. Fine at this size and easy to verify.
fn sort_i64(v: &mut Vec<i64>) {
    let mut i = 1;
    while i < v.len() {
        let x = v[i];
        let mut j = i;
        while j > 0 && v[j - 1] > x {
            v[j] = v[j - 1];
            j -= 1;
        }
        v[j] = x;
        i += 1;
    }
}

/// Nearest-rank percentile of an already sorted vector. pct is 0..100.
fn percentile(sorted: &Vec<i64>, pct: i64) -> i64 {
    if sorted.is_empty() {
        return 0;
    }
    let n = sorted.len() as i64;
    let mut rank = (pct * n + 99) / 100;
    if rank < 1 {
        rank = 1;
    }
    if rank > n {
        rank = n;
    }
    sorted[(rank - 1) as usize]
}

fn mean_i64(v: &Vec<i64>) -> i64 {
    if v.is_empty() {
        return 0;
    }
    let mut sum = 0;
    let mut i = 0;
    while i < v.len() {
        sum += v[i];
        i += 1;
    }
    sum / v.len() as i64
}

/// Share of `part` in `whole`, in permille. Zero when whole is zero.
fn permille(part: i64, whole: i64) -> i64 {
    if whole <= 0 {
        0
    } else {
        part * 1000 / whole
    }
}

#[derive(Debug, Clone)]
struct ServiceStats {
    service: String,
    count: i64,
    errors: i64,
    warnings: i64,
    timeouts: i64,
    latencies: Vec<i64>,
    slowest_req: u64,
    slowest_ms: i64,
}

fn empty_stats(service: &str) -> ServiceStats {
    ServiceStats {
        service: service.to_string(),
        count: 0,
        errors: 0,
        warnings: 0,
        timeouts: 0,
        latencies: Vec::new(),
        slowest_req: 0,
        slowest_ms: 0,
    }
}

fn stats_index(stats: &Vec<ServiceStats>, service: &str) -> Option<usize> {
    let mut i = 0;
    while i < stats.len() {
        if stats[i].service == service {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn collect_stats(lines: &Vec<LogLine>) -> Vec<ServiceStats> {
    let mut stats: Vec<ServiceStats> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].clone();
        let idx = match stats_index(&stats, &line.service) {
            Some(k) => k,
            None => {
                stats.push(empty_stats(&line.service));
                stats.len() - 1
            }
        };
        stats[idx].count += 1;
        if level_is_bad(line.level) {
            stats[idx].errors += 1;
        }
        if line.level == Level::Warn {
            stats[idx].warnings += 1;
        }
        if contains(&line.message, "timeout") {
            stats[idx].timeouts += 1;
        }
        if line.latency_ms >= 0 {
            stats[idx].latencies.push(line.latency_ms);
            if line.latency_ms > stats[idx].slowest_ms {
                stats[idx].slowest_ms = line.latency_ms;
                stats[idx].slowest_req = line.request_id;
            }
        }
        i += 1;
    }
    let mut j = 0;
    while j < stats.len() {
        let mut lat = stats[j].latencies.clone();
        sort_i64(&mut lat);
        stats[j].latencies = lat;
        j += 1;
    }
    stats
}

/// Sorts service stats by count, largest first (selection sort on indices is
/// overkill; swap whole structs, the list is short).
fn sort_stats_by_count(stats: &mut Vec<ServiceStats>) {
    let mut i = 0;
    while i < stats.len() {
        let mut best = i;
        let mut j = i + 1;
        while j < stats.len() {
            if stats[j].count > stats[best].count {
                best = j;
            }
            j += 1;
        }
        if best != i {
            let tmp = stats[i].clone();
            stats[i] = stats[best].clone();
            stats[best] = tmp;
        }
        i += 1;
    }
}

#[derive(Debug, Clone)]
struct HourBucket {
    hour: i64,
    count: i64,
    errors: i64,
}

fn hourly_buckets(lines: &Vec<LogLine>) -> Vec<HourBucket> {
    let mut out: Vec<HourBucket> = Vec::new();
    let mut h = 0;
    while h < 24 {
        out.push(HourBucket { hour: h, count: 0, errors: 0 });
        h += 1;
    }
    let mut i = 0;
    while i < lines.len() {
        let hour = (lines[i].minute / 60) as usize;
        if hour < 24 {
            out[hour].count += 1;
            if level_is_bad(lines[i].level) {
                out[hour].errors += 1;
            }
        }
        i += 1;
    }
    out
}

#[derive(Debug, Clone)]
struct LevelCount {
    level: Level,
    count: i64,
}

fn count_levels(lines: &Vec<LogLine>) -> Vec<LevelCount> {
    let mut out = vec![
        LevelCount { level: Level::Trace, count: 0 },
        LevelCount { level: Level::Debug, count: 0 },
        LevelCount { level: Level::Info, count: 0 },
        LevelCount { level: Level::Warn, count: 0 },
        LevelCount { level: Level::Error, count: 0 },
        LevelCount { level: Level::Fatal, count: 0 },
    ];
    let mut i = 0;
    while i < lines.len() {
        let r = level_rank(lines[i].level) as usize;
        out[r].count += 1;
        i += 1;
    }
    out
}

// ---------- Burst detection, slow requests, filters ----------

#[derive(Debug, Clone)]
struct Burst {
    service: String,
    start_minute: i64,
    end_minute: i64,
    errors: i64,
    total: i64,
}

/// Finds windows of `window` minutes where one service's error share reaches
/// `threshold_permille` with at least `min_errors` errors. Adjacent hot windows
/// merge into one burst.
fn find_bursts(lines: &Vec<LogLine>, services: &Vec<String>, window: i64, threshold_permille: i64, min_errors: i64) -> Vec<Burst> {
    let mut bursts: Vec<Burst> = Vec::new();
    let mut s = 0;
    while s < services.len() {
        let svc = services[s].clone();
        let mut minute = 0;
        let mut open: Option<Burst> = None;
        while minute < 24 * 60 {
            let mut total = 0;
            let mut errors = 0;
            let mut i = 0;
            while i < lines.len() {
                if lines[i].service == svc && lines[i].minute >= minute && lines[i].minute < minute + window {
                    total += 1;
                    if level_is_bad(lines[i].level) {
                        errors += 1;
                    }
                }
                i += 1;
            }
            let enough_errors = errors >= min_errors;
            let share_high = permille(errors, total) >= threshold_permille;
            let hot = if enough_errors { share_high } else { false };
            if hot {
                if let Some(b) = open.clone() {
                    let b_service = b.service.clone();
                    let b_start = b.start_minute;
                    let b_errors = b.errors + errors;
                    let b_total = b.total + total;
                    open = Some(Burst { service: b_service, start_minute: b_start, end_minute: minute + window, errors: b_errors, total: b_total });
                } else {
                    open = Some(Burst { service: svc.clone(), start_minute: minute, end_minute: minute + window, errors, total });
                }
            } else if let Some(b) = open.clone() {
                bursts.push(b);
                open = None;
            }
            minute += window;
        }
        if let Some(b) = open {
            bursts.push(b);
        }
        s += 1;
    }
    bursts
}

fn service_names(stats: &Vec<ServiceStats>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < stats.len() {
        out.push(stats[i].service.clone());
        i += 1;
    }
    out
}

/// The K slowest requests, slowest first. Repeated selection; K is small.
fn slowest_requests(lines: &Vec<LogLine>, k: i64) -> Vec<LogLine> {
    let mut picked: Vec<usize> = Vec::new();
    let mut out: Vec<LogLine> = Vec::new();
    let mut round = 0;
    while round < k {
        let mut best: Option<usize> = None;
        let mut i = 0;
        while i < lines.len() {
            if lines[i].latency_ms >= 0 && !index_picked(&picked, i) {
                if let Some(b) = best {
                    if lines[i].latency_ms > lines[b].latency_ms {
                        best = Some(i);
                    }
                } else {
                    best = Some(i);
                }
            }
            i += 1;
        }
        match best {
            Some(b) => {
                picked.push(b);
                out.push(lines[b].clone());
            }
            None => return out,
        }
        round += 1;
    }
    out
}

fn index_picked(picked: &Vec<usize>, i: usize) -> bool {
    let mut j = 0;
    while j < picked.len() {
        if picked[j] == i {
            return true;
        }
        j += 1;
    }
    false
}

/// A simple query: minimum level, optional service substring, optional
/// message substring (case-insensitive), and a minute range.
#[derive(Debug, Clone)]
struct Filter {
    min_level: Level,
    service_part: String,
    message_part: String,
    from_minute: i64,
    to_minute: i64,
}

fn filter_matches(f: &Filter, line: &LogLine) -> bool {
    if level_rank(line.level) < level_rank(f.min_level) {
        return false;
    }
    if line.minute < f.from_minute || line.minute > f.to_minute {
        return false;
    }
    if !f.service_part.is_empty() && !contains(&line.service, &f.service_part) {
        return false;
    }
    if !f.message_part.is_empty() {
        let hay = to_lower(&line.message);
        let needle = to_lower(&f.message_part);
        if !contains(&hay, &needle) {
            return false;
        }
    }
    true
}

fn apply_filter(lines: &Vec<LogLine>, f: &Filter) -> Vec<LogLine> {
    let mut out: Vec<LogLine> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if filter_matches(f, &lines[i]) {
            out.push(lines[i].clone());
        }
        i += 1;
    }
    out
}

/// Top message texts by frequency (first-seen order breaks ties by scan order).
#[derive(Debug, Clone)]
struct MessageCount {
    message: String,
    count: i64,
    errors: i64,
}

fn top_messages(lines: &Vec<LogLine>, k: i64) -> Vec<MessageCount> {
    let mut counts: Vec<MessageCount> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let mut at: Option<usize> = None;
        let mut j = 0;
        while j < counts.len() {
            if counts[j].message == lines[i].message {
                at = Some(j);
                break;
            }
            j += 1;
        }
        let idx = match at {
            Some(x) => x,
            None => {
                counts.push(MessageCount { message: lines[i].message.clone(), count: 0, errors: 0 });
                counts.len() - 1
            }
        };
        counts[idx].count += 1;
        if level_is_bad(lines[i].level) {
            counts[idx].errors += 1;
        }
        i += 1;
    }
    // Selection sort by count, descending.
    let mut a = 0;
    while a < counts.len() {
        let mut best = a;
        let mut b = a + 1;
        while b < counts.len() {
            if counts[b].count > counts[best].count {
                best = b;
            }
            b += 1;
        }
        if best != a {
            let tmp = counts[a].clone();
            counts[a] = counts[best].clone();
            counts[best] = tmp;
        }
        a += 1;
    }
    let mut out: Vec<MessageCount> = Vec::new();
    let mut t: i64 = 0;
    while t < k && (t as usize) < counts.len() {
        out.push(counts[t as usize].clone());
        t += 1;
    }
    out
}

// ---------- Report rendering ----------

fn render_header(title: &str) -> String {
    let mut s = String::from("\n== ");
    s.push_str(title);
    s.push_str(" ");
    s.push_str(&repeat_char('=', 60 - title.len() as i64));
    s.push('\n');
    s
}

fn render_service_table(stats: &Vec<ServiceStats>) -> String {
    let mut s = render_header("per-service summary");
    s.push_str("service      count  err%   p50ms  p90ms  p99ms  meanms  timeouts  slowest\n");
    let mut i = 0;
    while i < stats.len() {
        let st = stats[i].clone();
        let errp = fmt_permille(permille(st.errors, st.count));
        let row = format!(
            "{} {} {} {} {} {} {} {} req={} {}ms\n",
            pad_right(&st.service, 12),
            pad_left(&format!("{}", st.count), 6),
            pad_left(&errp, 5),
            pad_left(&format!("{}", percentile(&st.latencies, 50)), 6),
            pad_left(&format!("{}", percentile(&st.latencies, 90)), 6),
            pad_left(&format!("{}", percentile(&st.latencies, 99)), 6),
            pad_left(&format!("{}", mean_i64(&st.latencies)), 7),
            pad_left(&format!("{}", st.timeouts), 9),
            st.slowest_req,
            st.slowest_ms
        );
        s.push_str(&row);
        i += 1;
    }
    s
}

fn render_levels(levels: &Vec<LevelCount>, total: i64) -> String {
    let mut s = render_header("levels");
    let mut i = 0;
    while i < levels.len() {
        let pc = fmt_permille(permille(levels[i].count, total));
        let bar = repeat_char('#', permille(levels[i].count, total) / 25);
        s.push_str(&format!("{} {} ({}%) {}\n", pad_right(&level_name(levels[i].level), 6), pad_left(&format!("{}", levels[i].count), 6), pc, bar));
        i += 1;
    }
    s
}

fn render_hours(hours: &Vec<HourBucket>) -> String {
    let mut s = render_header("hourly volume (only busy hours)");
    let mut max = 1;
    let mut i = 0;
    while i < hours.len() {
        if hours[i].count > max {
            max = hours[i].count;
        }
        i += 1;
    }
    let mut j = 0;
    while j < hours.len() {
        if hours[j].count > 0 {
            let bar = repeat_char('#', hours[j].count * 48 / max);
            let errp = fmt_permille(permille(hours[j].errors, hours[j].count));
            s.push_str(&format!("{}:00 {} err {}% {}\n", pad2(hours[j].hour), pad_left(&format!("{}", hours[j].count), 5), errp, bar));
        }
        j += 1;
    }
    s
}

fn render_bursts(bursts: &Vec<Burst>) -> String {
    let mut s = render_header("error bursts (10 min windows, err >= 25.0%, at least 4 errors)");
    if bursts.is_empty() {
        s.push_str("none\n");
        return s;
    }
    let mut i = 0;
    while i < bursts.len() {
        let b = bursts[i].clone();
        let pc = fmt_permille(permille(b.errors, b.total));
        s.push_str(&format!(
            "{} {}..{} errors {}/{} ({}%)\n",
            pad_right(&b.service, 10),
            fmt_clock(b.start_minute),
            fmt_clock(b.end_minute),
            b.errors,
            b.total,
            pc
        ));
        i += 1;
    }
    s
}

fn render_slowest(slow: &Vec<LogLine>) -> String {
    let mut s = render_header("slowest requests");
    let mut i = 0;
    while i < slow.len() {
        let l = slow[i].clone();
        s.push_str(&format!(
            "{} {} req={} {}ms status={} {}\n",
            fmt_clock(l.minute),
            pad_right(&l.service, 10),
            l.request_id,
            pad_left(&format!("{}", l.latency_ms), 6),
            l.status,
            l.message
        ));
        i += 1;
    }
    s
}

fn render_top_messages(top: &Vec<MessageCount>) -> String {
    let mut s = render_header("top messages");
    let mut i = 0;
    while i < top.len() {
        s.push_str(&format!("{} (errors {}) {}\n", pad_left(&format!("{}", top[i].count), 5), top[i].errors, top[i].message));
        i += 1;
    }
    s
}

fn render_line(l: &LogLine) -> String {
    format!(
        "{} {} {} req={} ms={} status={} msg=\"{}\"",
        fmt_clock(l.minute),
        pad_right(&level_name(l.level), 5),
        pad_right(&l.service, 10),
        l.request_id,
        l.latency_ms,
        l.status,
        l.message
    )
}

/// Folds every headline number into one integer, so a test can compare the
/// whole analysis between two compilers with one line of output.
fn checksum(lines: &Vec<LogLine>, stats: &Vec<ServiceStats>, bursts: &Vec<Burst>) -> i64 {
    let mut acc: i64 = 0;
    let mut i = 0;
    while i < lines.len() {
        acc = acc.wrapping_add(lines[i].latency_ms.wrapping_mul(31));
        acc = acc.wrapping_add(level_rank(lines[i].level) * 7);
        acc = acc.wrapping_add(lines[i].minute);
        acc = acc.wrapping_add(lines[i].service.len() as i64);
        i += 1;
    }
    let mut j = 0;
    while j < stats.len() {
        acc = acc.wrapping_add(percentile(&stats[j].latencies, 99) * 3);
        acc = acc.wrapping_add(stats[j].errors * 11);
        j += 1;
    }
    let mut k = 0;
    while k < bursts.len() {
        acc = acc.wrapping_add(bursts[k].errors * 13 + bursts[k].start_minute);
        k += 1;
    }
    acc
}

fn main() {
    let raw = generate_log(20260115, 6000);
    let parsed = parse_log(&raw);
    let lines = parsed.lines.clone();
    let total = lines.len() as i64;

    println!("logscan report for 2026-01-15");
    println!("parsed {} lines, rejected {} damaged lines", total, parsed.rejected);
    if !parsed.first_error.is_empty() {
        println!("first parse problem: {}", parsed.first_error);
    }

    let mut stats = collect_stats(&lines);
    sort_stats_by_count(&mut stats);
    print!("{}", render_service_table(&stats));
    print!("{}", render_levels(&count_levels(&lines), total));
    print!("{}", render_hours(&hourly_buckets(&lines)));

    let bursts = find_bursts(&lines, &service_names(&stats), 10, 250, 4);
    print!("{}", render_bursts(&bursts));
    print!("{}", render_slowest(&slowest_requests(&lines, 8)));
    print!("{}", render_top_messages(&top_messages(&lines, 6)));

    // The incident view an on-call engineer would ask for first.
    let incident_filter = Filter {
        min_level: Level::Error,
        service_part: "checkout".to_string(),
        message_part: String::new(),
        from_minute: 9 * 60,
        to_minute: 10 * 60,
    };
    let hits = apply_filter(&lines, &incident_filter);
    println!("{}", render_header("checkout errors, 09:00..10:00 (first 5)"));
    let mut shown = 0;
    while shown < hits.len() && shown < 5 {
        println!("{}", render_line(&hits[shown]));
        shown += 1;
    }
    println!("checkout errors in window: {}", hits.len());

    let timeout_filter = Filter {
        min_level: Level::Trace,
        service_part: String::new(),
        message_part: "Timeout".to_string(),
        from_minute: 0,
        to_minute: 24 * 60,
    };
    println!("lines mentioning timeout: {}", apply_filter(&lines, &timeout_filter).len());
    println!("checksum {}", checksum(&lines, &stats, &bursts));
}








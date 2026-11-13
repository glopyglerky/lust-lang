//! Go `regexp` and `regexp/syntax` (go1.27.1 `src/regexp`), as far as
//! `regexp.Compile` and `(*Regexp).MatchString` need them, and the
//! `unicode.SimpleFold` they use (go1.27.1 `src/unicode`, Unicode 17.0.0).
//!
//! PORT: not ported, because they do not change a `MatchString` result:
//! the one-pass and backtrack engines (Go picks them only for speed; the
//! NFA gives the same answer), capture positions (`MatchString` runs with
//! `ncap == 0`), the machine pools, the parse tree printer
//! (`syntax.Regexp.String`), `CapNames` and the `[]byte` and
//! `io.RuneReader` inputs.
//!
//! PORT: Go strings may hold invalid UTF-8. A Rust `&str` cannot, so Go's
//! `ErrInvalidUTF8` checks (`checkUTF8`, `nextRune` errors) never fail here.
//!
//! `\p{Name}` and `\P{Name}` read the `unicode` category and script tables
//! in `unicode_tables.rs` (generated from go1.27.1, Unicode 17.0.0).

use crate::prelude::*;

use crate::gostd::errors::{self, GoError};

use self::syntax::{EmptyOp, Rune};

// Go: regexp/regexp.go:79 Regexp
/// Regexp is the representation of a compiled regular expression.
/// A Regexp is safe for concurrent use by multiple threads.
pub struct Regexp {
    /// as passed to Compile
    expr: String,
    /// compiled program
    prog: syntax::Prog,
    /// required prefix in unanchored matches
    prefix: String,
    /// first rune in prefix
    prefix_rune: Rune,
    /// empty-width conditions required at start of match
    cond: EmptyOp,
    /// minimum length of the input in bytes
    min_input_len: isize,
    /// whether regexp prefers leftmost-longest match
    longest: bool,
}

impl Regexp {
    // Go: regexp/regexp.go:102 String
    /// String returns the source text used to compile the regular expression.
    pub fn string(&self) -> &str {
        &self.expr
    }

    // Go: regexp/regexp.go:505 MatchString

    // Go: regexp/exec.go:513 doMatch

    // Go: regexp/exec.go:521 find
}

// Go: regexp/regexp.go:129 Compile

// Go: regexp/regexp.go:166 compile

// Go: regexp/regexp.go:267 minInputLen
/// minInputLen walks the regexp to find the minimum length of any matchable input.
fn min_input_len(nodes: &[syntax::Regexp], re: usize) -> isize {
    let re = &nodes[re];
    match re.op {
        syntax::OP_ANY_CHAR | syntax::OP_ANY_CHAR_NOT_NL | syntax::OP_CHAR_CLASS => 1,
        syntax::OP_LITERAL => {
            let mut l = 0;
            for &r in &re.rune {
                if r == UTF8_RUNE_ERROR {
                    l += 1;
                } else {
                    l += utf8_rune_len(r);
                }
            }
            l
        }
        syntax::OP_CAPTURE | syntax::OP_PLUS => min_input_len(nodes, re.sub[0]),
        syntax::OP_REPEAT => re.min as isize * min_input_len(nodes, re.sub[0]),
        syntax::OP_CONCAT => {
            let mut l = 0;
            for &sub in &re.sub {
                l += min_input_len(nodes, sub);
            }
            l
        }
        syntax::OP_ALTERNATE => {
            let mut l = min_input_len(nodes, re.sub[0]);
            for &sub in &re.sub[1..] {
                let lnext = min_input_len(nodes, sub);
                if lnext < l {
                    l = lnext;
                }
            }
            l
        }
        _ => 0,
    }
}

/// Go `utf8.RuneError`.
const UTF8_RUNE_ERROR: Rune = 0xFFFD;

/// Go `utf8.RuneLen`: the number of bytes in the UTF-8 encoding of r, or
/// -1 when r is not a valid rune.
fn utf8_rune_len(r: Rune) -> isize {
    match r {
        _ if r < 0 => -1,
        0..=0x7F => 1,
        0x80..=0x7FF => 2,
        0xD800..=0xDFFF => -1,
        0x800..=0xFFFF => 3,
        0x1_0000..=0x10_FFFF => 4,
        _ => -1,
    }
}

// Go: regexp/regexp.go:367 endOfText
const END_OF_TEXT: Rune = -1;

// Go: regexp/regexp.go:380 inputString
/// inputString scans a string.
struct InputString<'a> {
    str_: &'a str,
}

impl InputString<'_> {
    // Go: regexp/regexp.go:384 step
    fn step(&self, pos: usize) -> (Rune, usize) {
        if pos < self.str_.len() {
            return decode_rune_in_string(&self.str_[pos..]);
        }
        (END_OF_TEXT, 0)
    }

    // Go: regexp/regexp.go:391 canCheckPrefix
    fn can_check_prefix(&self) -> bool {
        true
    }

    // Go: regexp/regexp.go:399 index

    // Go: regexp/regexp.go:403 context
}

/// Go `utf8.DecodeRuneInString` on a valid string: the first rune and its
/// width, or (`RuneError`, 0) for an empty string.
fn decode_rune_in_string(s: &str) -> (Rune, usize) {
    match s.chars().next() {
        Some(c) => (c as Rune, c.len_utf8()),
        None => (UTF8_RUNE_ERROR, 0),
    }
}

// Go: regexp/exec.go:15 queue
/// A queue is a 'sparse array' holding pending threads of execution.
/// See https://research.swtch.com/2008/03/using-uninitialized-memory-for-fun-and.html
struct Queue {
    sparse: Vec<u32>,
    dense: Vec<Entry>,
}

// Go: regexp/exec.go:24 entry
/// An entry is an entry on a queue.
/// It holds both the instruction pc and the actual thread.
/// Some queue entries are just place holders so that the machine
/// knows it has considered that pc. Such entries have t == false.
// PORT: a Go thread is an instruction and a capture array. With no
// captures, the thread's instruction is always `pc`, so a flag is enough.
#[derive(Clone, Copy)]
struct Entry {
    pc: u32,
    t: bool,
}

// Go: regexp/exec.go:38 machine
/// A machine holds all the state during an NFA simulation for p.
struct Machine<'a> {
    /// corresponding Regexp
    re: &'a Regexp,
    /// compiled program
    p: &'a syntax::Prog,
    /// two queues for runq, nextq
    q0: Queue,
    q1: Queue,
    /// whether a match was found
    matched: bool,
}

// Go: regexp/exec.go:122 lazyFlag
/// A lazyFlag is a lazily-evaluated syntax.EmptyOp,
/// for checking zero-width flags like ^ $ \A \z \B \b.
/// It records the pair of relevant runes and does not
/// determine the implied flags until absolutely necessary
/// (most of the time, that means never).
#[derive(Clone, Copy)]
struct LazyFlag(u64);

// Go: regexp/exec.go:124 newLazyFlag
fn new_lazy_flag(r1: Rune, r2: Rune) -> LazyFlag {
    LazyFlag((i64::from(r1) as u64) << 32 | u64::from(r2 as u32))
}

impl LazyFlag {
    // Go: regexp/exec.go:128 match
    fn match_(self, mut op: EmptyOp) -> bool {
        if op == 0 {
            return true;
        }
        let r1 = (self.0 >> 32) as u32 as Rune;
        if op & syntax::EMPTY_BEGIN_LINE != 0 {
            if r1 != '\n' as Rune && r1 >= 0 {
                return false;
            }
            op &= !syntax::EMPTY_BEGIN_LINE;
        }
        if op & syntax::EMPTY_BEGIN_TEXT != 0 {
            if r1 >= 0 {
                return false;
            }
            op &= !syntax::EMPTY_BEGIN_TEXT;
        }
        if op == 0 {
            return true;
        }
        let r2 = self.0 as u32 as Rune;
        if op & syntax::EMPTY_END_LINE != 0 {
            if r2 != '\n' as Rune && r2 >= 0 {
                return false;
            }
            op &= !syntax::EMPTY_END_LINE;
        }
        if op & syntax::EMPTY_END_TEXT != 0 {
            if r2 >= 0 {
                return false;
            }
            op &= !syntax::EMPTY_END_TEXT;
        }
        if op == 0 {
            return true;
        }
        if syntax::is_word_char(r1) != syntax::is_word_char(r2) {
            op &= !syntax::EMPTY_WORD_BOUNDARY;
        } else {
            op &= !syntax::EMPTY_NO_WORD_BOUNDARY;
        }
        op == 0
    }
}

impl<'a> Machine<'a> {
    // Go: regexp/regexp.go:231 get
    /// get returns a machine to use for matching re.
    // PORT: Go takes the machine from a pool; the port makes a new one.
    fn new(re: &'a Regexp) -> Self {
        let n = re.prog.inst.len();
        Machine {
            re,
            p: &re.prog,
            q0: Queue {
                sparse: vec![0; n],
                dense: Vec::with_capacity(n),
            },
            q1: Queue {
                sparse: vec![0; n],
                dense: Vec::with_capacity(n),
            },
            matched: false,
        }
    }

    // Go: regexp/exec.go:175 match

    // Go: regexp/exec.go:260 step
    /// step executes one step of the machine, running each of the threads
    /// on runq and appending new threads to nextq.
    /// The step processes the rune c (which may be endOfText),
    /// which starts at position pos and ends at nextPos.
    /// nextCond gives the setting for the empty-width flags after c.
    fn step(
        &mut self,
        runq: &mut Queue,
        nextq: &mut Queue,
        _pos: usize,
        next_pos: usize,
        c: Rune,
        next_cond: LazyFlag,
    ) {
        let longest = self.re.longest;
        let p = self.p;
        let mut j = 0;
        while j < runq.dense.len() {
            let d = runq.dense[j];
            j += 1;
            if !d.t {
                continue;
            }
            // PORT: the `longest` thread cut needs capture slots, which
            // MatchString does not have.
            let i = &p.inst[d.pc as usize];
            let mut add = false;
            match i.op {
                syntax::INST_MATCH => {
                    if !longest {
                        // First-match mode: cut off all lower-priority threads.
                        runq.dense.clear();
                    }
                    self.matched = true;
                }
                syntax::INST_RUNE => add = i.match_rune(c),
                syntax::INST_RUNE1 => add = c == i.rune[0],
                syntax::INST_RUNE_ANY => add = true,
                syntax::INST_RUNE_ANY_NOT_NL => add = c != '\n' as Rune,
                _ => panic!("bad inst"),
            }
            if add {
                self.add(nextq, i.out, next_pos, next_cond);
            }
        }
        runq.dense.clear();
    }

    // Go: regexp/exec.go:317 add
    /// add adds an entry to q for pc, unless the q already has such an entry.
    /// It also recursively adds an entry for all instructions reachable from pc by following
    /// empty-width conditions satisfied by cond.  pos gives the current position
    /// in the input.
    // PORT: no capture slots, so InstCapture always follows Out, and the
    // thread passed back to the caller for reuse is dropped.
    fn add(&self, q: &mut Queue, mut pc: u32, pos: usize, cond: LazyFlag) {
        loop {
            if pc == 0 {
                return;
            }
            let j = q.sparse[pc as usize];
            if (j as usize) < q.dense.len() && q.dense[j as usize].pc == pc {
                return;
            }

            let j = q.dense.len();
            q.dense.push(Entry { pc, t: false });
            q.sparse[pc as usize] = j as u32;

            let i = &self.p.inst[pc as usize];
            match i.op {
                syntax::INST_FAIL => {
                    // nothing
                }
                syntax::INST_ALT | syntax::INST_ALT_MATCH => {
                    self.add(q, i.out, pos, cond);
                    pc = i.arg;
                    continue;
                }
                syntax::INST_EMPTY_WIDTH => {
                    if cond.match_(i.arg as EmptyOp) {
                        pc = i.out;
                        continue;
                    }
                }
                syntax::INST_NOP | syntax::INST_CAPTURE => {
                    pc = i.out;
                    continue;
                }
                syntax::INST_MATCH
                | syntax::INST_RUNE
                | syntax::INST_RUNE1
                | syntax::INST_RUNE_ANY
                | syntax::INST_RUNE_ANY_NOT_NL => {
                    q.dense[j].t = true;
                }
                _ => panic!("unhandled"),
            }
            return;
        }
    }
}

/// Go `regexp/syntax`.
pub mod syntax {
    use std::collections::HashMap;
    use std::fmt;
    use std::sync::LazyLock;

    use super::unicode;
    use crate::gostd::unicode_tables::{self, RangeTable, map_get};

    /// Go `rune`.
    pub type Rune = i32;

    /// Go `unicode.MaxRune`.
    pub const MAX_RUNE: Rune = 0x10_FFFF;

    // Go: regexp/syntax/parse.go:17 Error
    /// An Error describes a failure to parse a regular expression
    /// and gives the offending expression.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Error {
        pub code: ErrorCode,
        pub expr: String,
    }

    impl Error {
        fn new(code: ErrorCode, expr: &str) -> Self {
            Error {
                code,
                expr: expr.to_string(),
            }
        }

        // Go: regexp/syntax/parse.go:22 Error
        pub fn error(&self) -> String {
            format!(
                "error parsing regexp: {}: `{}`",
                self.code.string(),
                self.expr
            )
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.error())
        }
    }

    // Go: regexp/syntax/parse.go:27 ErrorCode
    /// An ErrorCode describes a failure to parse a regular expression.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ErrorCode(pub &'static str);

    impl ErrorCode {
        // Go: regexp/syntax/parse.go:51 String
        pub fn string(self) -> &'static str {
            self.0
        }
    }

    // Unexpected error
    pub const ERR_INTERNAL_ERROR: ErrorCode = ErrorCode("regexp/syntax: internal error");

    // Parse errors
    pub const ERR_INVALID_CHAR_CLASS: ErrorCode = ErrorCode("invalid character class");
    pub const ERR_INVALID_CHAR_RANGE: ErrorCode = ErrorCode("invalid character class range");
    pub const ERR_INVALID_ESCAPE: ErrorCode = ErrorCode("invalid escape sequence");
    pub const ERR_INVALID_NAMED_CAPTURE: ErrorCode = ErrorCode("invalid named capture");
    pub const ERR_INVALID_PERL_OP: ErrorCode = ErrorCode("invalid or unsupported Perl syntax");
    pub const ERR_INVALID_REPEAT_OP: ErrorCode = ErrorCode("invalid nested repetition operator");
    pub const ERR_INVALID_REPEAT_SIZE: ErrorCode = ErrorCode("invalid repeat count");
    pub const ERR_INVALID_UTF8: ErrorCode = ErrorCode("invalid UTF-8");
    pub const ERR_MISSING_BRACKET: ErrorCode = ErrorCode("missing closing ]");
    pub const ERR_MISSING_PAREN: ErrorCode = ErrorCode("missing closing )");
    pub const ERR_MISSING_REPEAT_ARGUMENT: ErrorCode =
        ErrorCode("missing argument to repetition operator");
    pub const ERR_TRAILING_BACKSLASH: ErrorCode =
        ErrorCode("trailing backslash at end of expression");
    pub const ERR_UNEXPECTED_PAREN: ErrorCode = ErrorCode("unexpected )");
    pub const ERR_NESTING_DEPTH: ErrorCode = ErrorCode("expression nests too deeply");
    pub const ERR_LARGE: ErrorCode = ErrorCode("expression too large");

    // Go: regexp/syntax/parse.go:56 Flags
    /// Flags control the behavior of the parser and record information about regexp context.
    pub type Flags = u16;

    /// case-insensitive match
    pub const FOLD_CASE: Flags = 1 << 0;
    /// treat pattern as literal string
    pub const LITERAL: Flags = 1 << 1;
    /// allow character classes like [^a-z] and [[:space:]] to match newline
    pub const CLASS_NL: Flags = 1 << 2;
    /// allow . to match newline
    pub const DOT_NL: Flags = 1 << 3;
    /// treat ^ and $ as only matching at beginning and end of text
    pub const ONE_LINE: Flags = 1 << 4;
    /// make repetition operators default to non-greedy
    pub const NON_GREEDY: Flags = 1 << 5;
    /// allow Perl extensions
    pub const PERL_X: Flags = 1 << 6;
    /// allow \p{Han}, \P{Han} for Unicode group and negation
    pub const UNICODE_GROUPS: Flags = 1 << 7;
    /// regexp OpEndText was $, not \z
    pub const WAS_DOLLAR: Flags = 1 << 8;
    /// regexp contains no counted repetition
    pub const SIMPLE: Flags = 1 << 9;

    pub const MATCH_NL: Flags = CLASS_NL | DOT_NL;

    /// as close to Perl as possible
    pub const PERL: Flags = CLASS_NL | ONE_LINE | PERL_X | UNICODE_GROUPS;
    /// POSIX syntax
    pub const POSIX: Flags = 0;

    // Go: regexp/syntax/regexp.go:34 Op
    /// An Op is a single regular expression operator.
    ///
    /// Operators are listed in precedence order, tightest binding to weakest.
    /// Character class operators are listed simplest to most complex
    /// (OpLiteral, OpCharClass, OpAnyCharNotNL, OpAnyChar).
    pub type Op = u8;

    /// matches no strings
    pub const OP_NO_MATCH: Op = 1;
    /// matches empty string
    pub const OP_EMPTY_MATCH: Op = 2;
    /// matches Runes sequence
    pub const OP_LITERAL: Op = 3;
    /// matches Runes interpreted as range pair list
    pub const OP_CHAR_CLASS: Op = 4;
    /// matches any character except newline
    pub const OP_ANY_CHAR_NOT_NL: Op = 5;
    /// matches any character
    pub const OP_ANY_CHAR: Op = 6;
    /// matches empty string at beginning of line
    pub const OP_BEGIN_LINE: Op = 7;
    /// matches empty string at end of line
    pub const OP_END_LINE: Op = 8;
    /// matches empty string at beginning of text
    pub const OP_BEGIN_TEXT: Op = 9;
    /// matches empty string at end of text
    pub const OP_END_TEXT: Op = 10;
    /// matches word boundary `\b`
    pub const OP_WORD_BOUNDARY: Op = 11;
    /// matches word non-boundary `\B`
    pub const OP_NO_WORD_BOUNDARY: Op = 12;
    /// capturing subexpression with index Cap, optional name Name
    pub const OP_CAPTURE: Op = 13;
    /// matches Sub[0] zero or more times
    pub const OP_STAR: Op = 14;
    /// matches Sub[0] one or more times
    pub const OP_PLUS: Op = 15;
    /// matches Sub[0] zero or one times
    pub const OP_QUEST: Op = 16;
    /// matches Sub[0] at least Min times, at most Max (Max == -1 is no limit)
    pub const OP_REPEAT: Op = 17;
    /// matches concatenation of Subs
    pub const OP_CONCAT: Op = 18;
    /// matches alternation of Subs
    pub const OP_ALTERNATE: Op = 19;

    /// where pseudo-ops start
    const OP_PSEUDO: Op = 128;

    // Pseudo-ops for parsing stack.
    const OP_LEFT_PAREN: Op = OP_PSEUDO;
    const OP_VERTICAL_BAR: Op = OP_PSEUDO + 1;

    // Go: regexp/syntax/regexp.go:17 Regexp
    /// A Regexp is a node in a regular expression syntax tree.
    // PORT: Go links nodes with pointers. The port keeps the nodes of one
    // regexp in an arena (`Vec<Regexp>`) and `sub` holds arena indexes, so
    // node identity (the parser's free list and its height and size maps)
    // is the index. `Sub0` and `Rune0` are Go storage for short slices.
    #[derive(Clone, Debug, Default)]
    pub struct Regexp {
        /// operator
        pub op: Op,
        pub flags: Flags,
        /// subexpressions, if any
        pub sub: Vec<usize>,
        /// matched runes, for OpLiteral, OpCharClass
        pub rune: Vec<Rune>,
        /// min, max for OpRepeat
        pub min: i32,
        pub max: i32,
        /// capturing index, for OpCapture
        pub cap: i32,
        /// capturing name, for OpCapture
        pub name: String,
    }

    // Go: regexp/syntax/regexp.go:64 Equal
    /// Equal reports whether x and y have identical structure.
    pub fn equal(nodes: &[Regexp], x: usize, y: usize) -> bool {
        let (xr, yr) = (&nodes[x], &nodes[y]);
        if xr.op != yr.op {
            return false;
        }
        match xr.op {
            OP_END_TEXT => {
                // The parse flags remember whether this is \z or \Z.
                if xr.flags & WAS_DOLLAR != yr.flags & WAS_DOLLAR {
                    return false;
                }
            }
            OP_LITERAL | OP_CHAR_CLASS => {
                return xr.flags & FOLD_CASE == yr.flags & FOLD_CASE && xr.rune == yr.rune;
            }
            OP_ALTERNATE | OP_CONCAT => {
                return xr.sub.len() == yr.sub.len()
                    && xr
                        .sub
                        .iter()
                        .zip(&yr.sub)
                        .all(|(&a, &b)| equal(nodes, a, b));
            }
            OP_STAR | OP_PLUS | OP_QUEST => {
                if xr.flags & NON_GREEDY != yr.flags & NON_GREEDY
                    || !equal(nodes, xr.sub[0], yr.sub[0])
                {
                    return false;
                }
            }
            OP_REPEAT => {
                if xr.flags & NON_GREEDY != yr.flags & NON_GREEDY
                    || xr.min != yr.min
                    || xr.max != yr.max
                    || !equal(nodes, xr.sub[0], yr.sub[0])
                {
                    return false;
                }
            }
            OP_CAPTURE => {
                if xr.cap != yr.cap || xr.name != yr.name || !equal(nodes, xr.sub[0], yr.sub[0]) {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    /// maxHeight is the maximum height of a regexp parse tree.
    /// It is somewhat arbitrarily chosen, but the idea is to be large enough
    /// that no one will actually hit in real use but at the same time small enough
    /// that recursion on the Regexp tree will not hit the 1GB Go stack limit.
    /// As an optimization, we don't even bother calculating heights
    /// until we've allocated at least maxHeight Regexp structures.
    const MAX_HEIGHT: usize = 1000;

    /// maxSize is the maximum size of a compiled regexp in Insts.
    /// It too is somewhat arbitrarily chosen, but the idea is to be large enough
    /// to allow significant regexps while at the same time small enough that
    /// the compiled form will not take up too much memory.
    /// 128 MB is enough for a 3.3 million Inst structures, which roughly
    /// corresponds to a 3.3 MB regexp.
    const MAX_SIZE: i64 = (128 << 20) / INST_SIZE;
    /// byte, 2 uint32, slice is 5 64-bit words
    const INST_SIZE: i64 = 5 * 8;

    /// maxRunes is the maximum number of runes allowed in a regexp tree
    /// counting the runes in all the nodes.
    /// Ignoring character classes p.numRunes is always less than the length of the regexp.
    /// Character classes can make it much larger: each \pL adds 1292 runes.
    const MAX_RUNES: usize = (128 << 20) / RUNE_SIZE;
    /// rune is int32
    const RUNE_SIZE: usize = 4;

    // Go: regexp/syntax/parse.go:127 parser
    struct Parser {
        /// parse mode flags
        flags: Flags,
        /// stack of parsed expressions
        stack: Vec<usize>,
        // PORT: Go threads the free list through `Sub0[0]`; a stack gives
        // the same last-in first-out order.
        free: Vec<usize>,
        /// number of capturing groups seen
        num_cap: i32,
        whole_regexp: String,
        /// number of regexps allocated
        num_regexp: usize,
        /// number of runes in char classes
        num_runes: usize,
        /// product of all repetitions seen
        repeats: i64,
        /// regexp height, for height limit check
        height: Option<HashMap<usize, usize>>,
        /// regexp compiled size, for size limit check
        size: Option<HashMap<usize, i64>>,
        /// the node arena
        nodes: Vec<Regexp>,
    }

    impl Parser {
        // Go: regexp/syntax/parse.go:141 newRegexp
        fn new_regexp(&mut self, op: Op) -> usize {
            let re = if let Some(re) = self.free.pop() {
                self.nodes[re] = Regexp::default();
                re
            } else {
                self.nodes.push(Regexp::default());
                self.num_regexp += 1;
                self.nodes.len() - 1
            };
            self.nodes[re].op = op;
            re
        }

        // Go: regexp/syntax/parse.go:154 reuse
        fn reuse(&mut self, re: usize) {
            if let Some(height) = &mut self.height {
                height.remove(&re);
            }
            self.free.push(re);
        }

        // Go: regexp/syntax/parse.go:162 checkLimits
        // PORT: Go panics with ErrLarge or ErrNestingDepth and `parse`
        // recovers it as `&Error{code, s}`. The port returns that error.
        fn check_limits(&mut self, re: usize) -> Result<(), Error> {
            if self.num_runes > MAX_RUNES {
                return Err(Error::new(ERR_LARGE, &self.whole_regexp));
            }
            self.check_size(re)?;
            self.check_height(re)
        }

        // Go: regexp/syntax/parse.go:170 checkSize
        fn check_size(&mut self, re: usize) -> Result<(), Error> {
            if self.size.is_none() {
                // We haven't started tracking size yet.
                // Do a relatively cheap check to see if we need to start.
                // Maintain the product of all the repeats we've seen
                // and don't track if the total number of regexp nodes
                // we've seen times the repeat product is in budget.
                if self.repeats == 0 {
                    self.repeats = 1;
                }
                if self.nodes[re].op == OP_REPEAT {
                    let mut n = self.nodes[re].max;
                    if n == -1 {
                        n = self.nodes[re].min;
                    }
                    if n <= 0 {
                        n = 1;
                    }
                    if i64::from(n) > MAX_SIZE / self.repeats {
                        self.repeats = MAX_SIZE;
                    } else {
                        self.repeats *= i64::from(n);
                    }
                }
                if (self.num_regexp as i64) < MAX_SIZE / self.repeats {
                    return Ok(());
                }

                // We need to start tracking size.
                // Make the map and belatedly populate it
                // with info about everything we've constructed so far.
                self.size = Some(HashMap::new());
                for re in self.stack.clone() {
                    self.check_size(re)?;
                }
            }

            if self.calc_size(re, true) > MAX_SIZE {
                return Err(Error::new(ERR_LARGE, &self.whole_regexp));
            }
            Ok(())
        }

        // Go: regexp/syntax/parse.go:212 calcSize
        fn calc_size(&mut self, re: usize, force: bool) -> i64 {
            if !force {
                if let Some(&size) = self.size.as_ref().and_then(|m| m.get(&re)) {
                    return size;
                }
            }

            let mut size: i64 = 0;
            let (op, min, max) = (self.nodes[re].op, self.nodes[re].min, self.nodes[re].max);
            match op {
                OP_LITERAL => size = self.nodes[re].rune.len() as i64,
                OP_CAPTURE | OP_STAR => {
                    // star can be 1+ or 2+; assume 2 pessimistically
                    size = 2 + self.calc_size(self.nodes[re].sub[0], false);
                }
                OP_PLUS | OP_QUEST => size = 1 + self.calc_size(self.nodes[re].sub[0], false),
                OP_CONCAT => {
                    for sub in self.nodes[re].sub.clone() {
                        size += self.calc_size(sub, false);
                    }
                }
                OP_ALTERNATE => {
                    let subs = self.nodes[re].sub.clone();
                    for &sub in &subs {
                        size += self.calc_size(sub, false);
                    }
                    if subs.len() > 1 {
                        size += subs.len() as i64 - 1;
                    }
                }
                OP_REPEAT => {
                    let sub = self.calc_size(self.nodes[re].sub[0], false);
                    if max == -1 {
                        if min == 0 {
                            size = 2 + sub; // x*
                        } else {
                            size = 1 + i64::from(min) * sub; // xxx+
                        }
                    } else {
                        // x{2,5} = xx(x(x(x)?)?)?
                        size = i64::from(max) * sub + i64::from(max - min);
                    }
                }
                _ => {}
            }

            size = size.max(1);
            if let Some(m) = &mut self.size {
                m.insert(re, size);
            }
            size
        }

        // Go: regexp/syntax/parse.go:258 checkHeight
        fn check_height(&mut self, re: usize) -> Result<(), Error> {
            if self.num_regexp < MAX_HEIGHT {
                return Ok(());
            }
            if self.height.is_none() {
                self.height = Some(HashMap::new());
                for re in self.stack.clone() {
                    self.check_height(re)?;
                }
            }
            if self.calc_height(re, true) > MAX_HEIGHT {
                return Err(Error::new(ERR_NESTING_DEPTH, &self.whole_regexp));
            }
            Ok(())
        }

        // Go: regexp/syntax/parse.go:273 calcHeight
        fn calc_height(&mut self, re: usize, force: bool) -> usize {
            if !force {
                if let Some(&h) = self.height.as_ref().and_then(|m| m.get(&re)) {
                    return h;
                }
            }
            let mut h = 1;
            for sub in self.nodes[re].sub.clone() {
                let hsub = self.calc_height(sub, false);
                if h < 1 + hsub {
                    h = 1 + hsub;
                }
            }
            if let Some(m) = &mut self.height {
                m.insert(re, h);
            }
            h
        }

        // Parse stack manipulation.

        // Go: regexp/syntax/parse.go:293 push
        /// push pushes the regexp re onto the parse stack and returns the regexp.
        /// It returns None when re was merged into the literal on top of the stack.
        fn push(&mut self, re: usize) -> Result<Option<usize>, Error> {
            self.num_runes += self.nodes[re].rune.len();
            let node = &self.nodes[re];
            let rune = &node.rune;
            if node.op == OP_CHAR_CLASS && rune.len() == 2 && rune[0] == rune[1] {
                // Single rune.
                let r0 = rune[0];
                if self.maybe_concat(r0, self.flags & !FOLD_CASE) {
                    return Ok(None);
                }
                let node = &mut self.nodes[re];
                node.op = OP_LITERAL;
                node.rune.truncate(1);
                node.flags = self.flags & !FOLD_CASE;
            } else if node.op == OP_CHAR_CLASS
                && rune.len() == 4
                && rune[0] == rune[1]
                && rune[2] == rune[3]
                && unicode::simple_fold(rune[0]) == rune[2]
                && unicode::simple_fold(rune[2]) == rune[0]
                || node.op == OP_CHAR_CLASS
                    && rune.len() == 2
                    && rune[0] + 1 == rune[1]
                    && unicode::simple_fold(rune[0]) == rune[1]
                    && unicode::simple_fold(rune[1]) == rune[0]
            {
                // Case-insensitive rune like [Aa] or [Δδ].
                let r0 = rune[0];
                if self.maybe_concat(r0, self.flags | FOLD_CASE) {
                    return Ok(None);
                }

                // Rewrite as (case-insensitive) literal.
                let node = &mut self.nodes[re];
                node.op = OP_LITERAL;
                node.rune.truncate(1);
                node.flags = self.flags | FOLD_CASE;
            } else {
                // Incremental concatenation.
                self.maybe_concat(-1, 0);
            }

            self.stack.push(re);
            self.check_limits(re)?;
            Ok(Some(re))
        }

        // Go: regexp/syntax/parse.go:339 maybeConcat
        /// maybeConcat implements incremental concatenation
        /// of literal runes into string nodes. The parser calls this
        /// before each push, so only the top fragment of the stack
        /// might need processing. Since this is called before a push,
        /// the topmost literal is no longer subject to operators like *
        /// (Otherwise ab* would turn into (ab)*.)
        /// If r >= 0 and there's a node left over, maybeConcat uses it
        /// to push r with the given flags.
        /// maybeConcat reports whether r was pushed.
        fn maybe_concat(&mut self, r: Rune, flags: Flags) -> bool {
            let n = self.stack.len();
            if n < 2 {
                return false;
            }

            let re1 = self.stack[n - 1];
            let re2 = self.stack[n - 2];
            if self.nodes[re1].op != OP_LITERAL
                || self.nodes[re2].op != OP_LITERAL
                || self.nodes[re1].flags & FOLD_CASE != self.nodes[re2].flags & FOLD_CASE
            {
                return false;
            }

            // Push re1 into re2.
            let re1_rune = self.nodes[re1].rune.clone();
            self.nodes[re2].rune.extend_from_slice(&re1_rune);

            // Reuse re1 if possible.
            if r >= 0 {
                self.nodes[re1].rune = vec![r];
                self.nodes[re1].flags = flags;
                return true;
            }

            self.stack.truncate(n - 1);
            self.reuse(re1);
            false // did not push r
        }

        // Go: regexp/syntax/parse.go:368 literal
        /// literal pushes a literal regexp for the rune r on the stack.
        fn literal(&mut self, mut r: Rune) -> Result<(), Error> {
            let re = self.new_regexp(OP_LITERAL);
            self.nodes[re].flags = self.flags;
            if self.flags & FOLD_CASE != 0 {
                r = min_fold_rune(r);
            }
            self.nodes[re].rune = vec![r];
            self.push(re)?;
            Ok(())
        }

        // Go: regexp/syntax/parse.go:394 op
        /// op pushes a regexp with the given op onto the stack
        /// and returns that regexp.
        fn op(&mut self, op: Op) -> Result<usize, Error> {
            let re = self.new_regexp(op);
            self.nodes[re].flags = self.flags;
            // PORT: push returns nil only for a char class, which op never gets.
            Ok(self.push(re)?.expect("op pushes a non-class regexp"))
        }

        // Go: regexp/syntax/parse.go:404 repeat
        /// repeat replaces the top stack element with itself repeated according to op, min, max.
        /// before is the regexp suffix starting at the repetition operator.
        /// after is the regexp suffix following after the repetition operator.
        /// repeat returns an updated 'after' and an error, if any.
        fn repeat<'a>(
            &mut self,
            op: Op,
            min: i32,
            max: i32,
            before: &'a str,
            after: &'a str,
            last_repeat: &'a str,
        ) -> Result<&'a str, Error> {
            let mut after = after;
            let mut flags = self.flags;
            if self.flags & PERL_X != 0 {
                if !after.is_empty() && after.as_bytes()[0] == b'?' {
                    after = &after[1..];
                    flags ^= NON_GREEDY;
                }
                if !last_repeat.is_empty() {
                    // In Perl it is not allowed to stack repetition operators:
                    // a** is a syntax error, not a doubled star, and a++ means
                    // something else entirely, which we don't support!
                    return Err(Error::new(
                        ERR_INVALID_REPEAT_OP,
                        &last_repeat[..last_repeat.len() - after.len()],
                    ));
                }
            }
            let n = self.stack.len();
            if n == 0 {
                return Err(Error::new(
                    ERR_MISSING_REPEAT_ARGUMENT,
                    &before[..before.len() - after.len()],
                ));
            }
            let sub = self.stack[n - 1];
            if self.nodes[sub].op >= OP_PSEUDO {
                return Err(Error::new(
                    ERR_MISSING_REPEAT_ARGUMENT,
                    &before[..before.len() - after.len()],
                ));
            }

            let re = self.new_regexp(op);
            let node = &mut self.nodes[re];
            node.min = min;
            node.max = max;
            node.flags = flags;
            node.sub = vec![sub];
            self.stack[n - 1] = re;
            self.check_limits(re)?;

            if op == OP_REPEAT && (min >= 2 || max >= 2) && !repeat_is_valid(&self.nodes, re, 1000)
            {
                return Err(Error::new(
                    ERR_INVALID_REPEAT_SIZE,
                    &before[..before.len() - after.len()],
                ));
            }

            Ok(after)
        }

        // Go: regexp/syntax/parse.go:477 concat

        // Go: regexp/syntax/parse.go:497 alternate

        // Go: regexp/syntax/parse.go:549 collapse

        // Go: regexp/syntax/parse.go:589 factor

        // Go: regexp/syntax/parse.go:778 leadingString
        /// leadingString returns the leading literal string that re begins with.
        // PORT: Go returns a slice of re's storage; the port returns a copy.
        fn leading_string(&self, mut re: usize) -> (Vec<Rune>, Flags) {
            if self.nodes[re].op == OP_CONCAT && !self.nodes[re].sub.is_empty() {
                re = self.nodes[re].sub[0];
            }
            if self.nodes[re].op != OP_LITERAL {
                return (Vec::new(), 0);
            }
            (
                self.nodes[re].rune.clone(),
                self.nodes[re].flags & FOLD_CASE,
            )
        }

        // Go: regexp/syntax/parse.go:790 removeLeadingString
        /// removeLeadingString removes the first n leading runes
        /// from the beginning of re. It returns the replacement for re.
        fn remove_leading_string(&mut self, mut re: usize, n: usize) -> usize {
            if self.nodes[re].op == OP_CONCAT && !self.nodes[re].sub.is_empty() {
                // Removing a leading string in a concatenation
                // might simplify the concatenation.
                let sub = self.remove_leading_string(self.nodes[re].sub[0], n);
                self.nodes[re].sub[0] = sub;
                if self.nodes[sub].op == OP_EMPTY_MATCH {
                    self.reuse(sub);
                    match self.nodes[re].sub.len() {
                        0 | 1 => {
                            // Impossible but handle.
                            self.nodes[re].op = OP_EMPTY_MATCH;
                            self.nodes[re].sub = Vec::new();
                        }
                        2 => {
                            let old = re;
                            re = self.nodes[re].sub[1];
                            self.reuse(old);
                        }
                        _ => {
                            self.nodes[re].sub.remove(0);
                        }
                    }
                }
                return re;
            }

            if self.nodes[re].op == OP_LITERAL {
                self.nodes[re].rune.drain(..n);
                if self.nodes[re].rune.is_empty() {
                    self.nodes[re].op = OP_EMPTY_MATCH;
                }
            }
            re
        }

        // Go: regexp/syntax/parse.go:827 leadingRegexp
        /// leadingRegexp returns the leading regexp that re begins with.
        /// The regexp refers to storage in re or its children.
        fn leading_regexp(&self, re: usize) -> Option<usize> {
            if self.nodes[re].op == OP_EMPTY_MATCH {
                return None;
            }
            if self.nodes[re].op == OP_CONCAT && !self.nodes[re].sub.is_empty() {
                let sub = self.nodes[re].sub[0];
                if self.nodes[sub].op == OP_EMPTY_MATCH {
                    return None;
                }
                return Some(sub);
            }
            Some(re)
        }

        // Go: regexp/syntax/parse.go:844 removeLeadingRegexp
        /// removeLeadingRegexp removes the leading regexp in re.
        /// It returns the replacement for re.
        /// If reuse is true, it passes the removed regexp (if no longer needed) to p.reuse.
        fn remove_leading_regexp(&mut self, re: usize, reuse: bool) -> usize {
            if self.nodes[re].op == OP_CONCAT && !self.nodes[re].sub.is_empty() {
                if reuse {
                    self.reuse(self.nodes[re].sub[0]);
                }
                self.nodes[re].sub.remove(0);
                match self.nodes[re].sub.len() {
                    0 => {
                        self.nodes[re].op = OP_EMPTY_MATCH;
                        self.nodes[re].sub = Vec::new();
                    }
                    1 => {
                        let old = re;
                        let re = self.nodes[re].sub[0];
                        self.reuse(old);
                        return re;
                    }
                    _ => {}
                }
                return re;
            }
            if reuse {
                self.reuse(re);
            }
            self.new_regexp(OP_EMPTY_MATCH)
        }

        // Go: regexp/syntax/parse.go:1102 parseRepeat

        // Go: regexp/syntax/parse.go:1141 parsePerlFlags

        // Go: regexp/syntax/parse.go:1330 parseVerticalBar

        // Go: regexp/syntax/parse.go:1375 swapVerticalBar

        // Go: regexp/syntax/parse.go:1411 parseRightParen

        // Go: regexp/syntax/parse.go:1445 parseEscape
        /// parseEscape parses an escape sequence at the beginning of s
        /// and returns the rune.
        fn parse_escape<'a>(&self, s: &'a str) -> Result<(Rune, &'a str), Error> {
            let mut t = &s[1..];
            if t.is_empty() {
                return Err(Error::new(ERR_TRAILING_BACKSLASH, ""));
            }
            let c;
            (c, t) = next_rune(t);

            'switch: {
                match c {
                    // Octal escapes.
                    '1'..='7' | '0' => {
                        // Single non-zero digit is a backreference; not supported
                        if c != '0'
                            && (t.is_empty() || t.as_bytes()[0] < b'0' || t.as_bytes()[0] > b'7')
                        {
                            break 'switch;
                        }
                        // Consume up to three octal digits; already have one.
                        let mut r = c as Rune - '0' as Rune;
                        for _ in 1..3 {
                            if t.is_empty() || t.as_bytes()[0] < b'0' || t.as_bytes()[0] > b'7' {
                                break;
                            }
                            r = r * 8 + Rune::from(t.as_bytes()[0]) - '0' as Rune;
                            t = &t[1..];
                        }
                        return Ok((r, t));
                    }

                    // Hexadecimal escapes.
                    'x' => {
                        if t.is_empty() {
                            break 'switch;
                        }
                        let mut c;
                        (c, t) = next_rune(t);
                        if c == '{' {
                            // Any number of digits in braces.
                            // Perl accepts any text at all; it ignores all text
                            // after the first non-hex digit. We require only hex digits,
                            // and at least one.
                            let mut nhex = 0;
                            let mut r: Rune = 0;
                            loop {
                                if t.is_empty() {
                                    break 'switch;
                                }
                                (c, t) = next_rune(t);
                                if c == '}' {
                                    break;
                                }
                                let v = unhex(c);
                                if v < 0 {
                                    break 'switch;
                                }
                                r = r * 16 + v;
                                if r > MAX_RUNE {
                                    break 'switch;
                                }
                                nhex += 1;
                            }
                            if nhex == 0 {
                                break 'switch;
                            }
                            return Ok((r, t));
                        }

                        // Easy case: two hex digits.
                        let x = unhex(c);
                        (c, t) = next_rune(t);
                        let y = unhex(c);
                        if x < 0 || y < 0 {
                            break 'switch;
                        }
                        return Ok((x * 16 + y, t));
                    }

                    // C escapes. There is no case 'b', to avoid misparsing
                    // the Perl word-boundary \b as the C backspace \b
                    // when in POSIX mode. In Perl, /\b/ means word-boundary
                    // but /[\b]/ means backspace. We don't support that.
                    // If you want a backspace, embed a literal backspace
                    // character or use \x08.
                    'a' => return Ok((7, t)),
                    'f' => return Ok((12, t)),
                    'n' => return Ok((10, t)),
                    'r' => return Ok((13, t)),
                    't' => return Ok((9, t)),
                    'v' => return Ok((11, t)),

                    _ => {
                        if c.is_ascii() && !isalnum(c) {
                            // Escaped non-word characters are always themselves.
                            // PCRE is not quite so rigorous: it accepts things like
                            // \q, but we don't. We once rejected \_, but too many
                            // programs and people insist on using it, so allow \_.
                            return Ok((c as Rune, t));
                        }
                    }
                }
            }
            Err(Error::new(ERR_INVALID_ESCAPE, &s[..s.len() - t.len()]))
        }

        // Go: regexp/syntax/parse.go:1561 parseClassChar
        /// parseClassChar parses a character class character at the beginning of s
        /// and returns it.
        fn parse_class_char<'a>(
            &self,
            s: &'a str,
            whole_class: &str,
        ) -> Result<(Rune, &'a str), Error> {
            if s.is_empty() {
                return Err(Error::new(ERR_MISSING_BRACKET, whole_class));
            }

            // Allow regular escape sequences even though
            // many need not be escaped in this context.
            if s.as_bytes()[0] == b'\\' {
                return self.parse_escape(s);
            }

            let (c, t) = next_rune(s);
            Ok((c as Rune, t))
        }

        // Go: regexp/syntax/parse.go:1585 parsePerlClassEscape

        // Go: regexp/syntax/parse.go:1599 parseNamedClass

        // Go: regexp/syntax/parse.go:1617 appendGroup
        // PORT: Go keeps a scratch buffer in p.tmpClass; the port uses a new Vec.

        // Go: regexp/syntax/parse.go:1760 parseUnicodeClass

        // Go: regexp/syntax/parse.go:1836 parseClass
    }

    // Go: regexp/syntax/parse.go:380 minFoldRune
    /// minFoldRune returns the minimum rune fold-equivalent to r.
    fn min_fold_rune(r: Rune) -> Rune {
        if !(MIN_FOLD..=MAX_FOLD).contains(&r) {
            return r;
        }
        let mut m = r;
        let r0 = r;
        let mut r = unicode::simple_fold(r);
        while r != r0 {
            m = m.min(r);
            r = unicode::simple_fold(r);
        }
        m
    }

    // Go: regexp/syntax/parse.go:452 repeatIsValid
    /// repeatIsValid reports whether the repetition re is valid.
    /// Valid means that the combination of the top-level repetition
    /// and any inner repetitions does not exceed n copies of the
    /// innermost thing.
    /// This function rewalks the regexp tree and is called for every repetition,
    /// so we have to worry about inducing quadratic behavior in the parser.
    /// We avoid this by only calling repeatIsValid when min or max >= 2.
    /// In that case the depth of any >= 2 nesting can only get to 9 without
    /// triggering a parse error, so each subtree can only be rewalked 9 times.
    fn repeat_is_valid(nodes: &[Regexp], re: usize, mut n: i32) -> bool {
        let node = &nodes[re];
        if node.op == OP_REPEAT {
            let mut m = node.max;
            if m == 0 {
                return true;
            }
            if m < 0 {
                m = node.min;
            }
            if m > n {
                return false;
            }
            if m > 0 {
                n /= m;
            }
        }
        for &sub in &node.sub {
            if !repeat_is_valid(nodes, sub, n) {
                return false;
            }
        }
        true
    }

    // Go: regexp/syntax/parse.go:523 cleanAlt

    // Go: regexp/syntax/parse.go:867 literalRegexp
    fn literal_regexp(s: &str, flags: Flags) -> Regexp {
        Regexp {
            op: OP_LITERAL,
            flags,
            rune: s.chars().map(|c| c as Rune).collect(),
            ..Regexp::default()
        }
    }

    // Parsing.

    // Go: regexp/syntax/parse.go:887 Parse

    // Go: regexp/syntax/parse.go:891 parse

    // Go: regexp/syntax/parse.go:1260 isValidCaptureName
    /// isValidCaptureName reports whether name
    /// is a valid capture name: [A-Za-z0-9_]+.
    /// PCRE limits names to 32 bytes.
    /// Python rejects names starting with digits.
    /// We don't enforce either of those.
    fn is_valid_capture_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        name.chars().all(|c| c == '_' || isalnum(c))
    }

    // Go: regexp/syntax/parse.go:1273 parseInt

    // Go: regexp/syntax/parse.go:1302 isCharClass
    /// can this be represented as a character class?
    /// single-rune literal string, char class, ., and .|\n.
    fn is_char_class(re: &Regexp) -> bool {
        re.op == OP_LITERAL && re.rune.len() == 1
            || re.op == OP_CHAR_CLASS
            || re.op == OP_ANY_CHAR_NOT_NL
            || re.op == OP_ANY_CHAR
    }

    // Go: regexp/syntax/parse.go:1310 matchRune

    // Go: regexp/syntax/parse.go:1345 mergeCharClass

    // Go: regexp/syntax/parse.go:1571 charGroup
    #[derive(Clone, Copy)]
    struct CharGroup {
        sign: i32,
        class: &'static [Rune],
    }

    // Go: regexp/syntax/perl_groups.go
    const CODE1: &[Rune] = &[/* \d */ 0x30, 0x39];
    const CODE2: &[Rune] = &[/* \s */ 0x9, 0xa, 0xc, 0xd, 0x20, 0x20];
    const CODE3: &[Rune] = &[/* \w */ 0x30, 0x39, 0x41, 0x5a, 0x5f, 0x5f, 0x61, 0x7a];
    const CODE4: &[Rune] = &[/* [:alnum:] */ 0x30, 0x39, 0x41, 0x5a, 0x61, 0x7a];
    const CODE5: &[Rune] = &[/* [:alpha:] */ 0x41, 0x5a, 0x61, 0x7a];
    const CODE6: &[Rune] = &[/* [:ascii:] */ 0x0, 0x7f];
    const CODE7: &[Rune] = &[/* [:blank:] */ 0x9, 0x9, 0x20, 0x20];
    const CODE8: &[Rune] = &[/* [:cntrl:] */ 0x0, 0x1f, 0x7f, 0x7f];
    const CODE9: &[Rune] = &[/* [:digit:] */ 0x30, 0x39];
    const CODE10: &[Rune] = &[/* [:graph:] */ 0x21, 0x7e];
    const CODE11: &[Rune] = &[/* [:lower:] */ 0x61, 0x7a];
    const CODE12: &[Rune] = &[/* [:print:] */ 0x20, 0x7e];
    const CODE13: &[Rune] = &[
        /* [:punct:] */ 0x21, 0x2f, 0x3a, 0x40, 0x5b, 0x60, 0x7b, 0x7e,
    ];
    const CODE14: &[Rune] = &[/* [:space:] */ 0x9, 0xd, 0x20, 0x20];
    const CODE15: &[Rune] = &[/* [:upper:] */ 0x41, 0x5a];
    const CODE16: &[Rune] = &[
        /* [:word:] */ 0x30, 0x39, 0x41, 0x5a, 0x5f, 0x5f, 0x61, 0x7a,
    ];
    const CODE17: &[Rune] = &[/* [:xdigit:] */ 0x30, 0x39, 0x41, 0x46, 0x61, 0x66];

    // Go: regexp/syntax/perl_groups.go perlGroup
    // PORT: the Go map lookup is a match on the key.

    // Go: regexp/syntax/perl_groups.go posixGroup
    // PORT: the Go map lookup is a match on the key.

    // Go: regexp/syntax/parse.go:1638 anyTable
    static ANY_TABLE: RangeTable = RangeTable {
        r16: &[(0, u16::MAX, 1)], // 1<<16 - 1
        r32: &[(1 << 16, MAX_RUNE as u32, 1)],
        latin_offset: 0,
    };

    // Go: regexp/syntax/parse.go:1643 asciiTable
    static ASCII_TABLE: RangeTable = RangeTable {
        r16: &[(0, 0x7F, 1)],
        r32: &[],
        latin_offset: 0,
    };

    // Go: regexp/syntax/parse.go:1647 asciiFoldTable
    static ASCII_FOLD_TABLE: RangeTable = RangeTable {
        r16: &[
            (0, 0x7F, 1),
            (0x017F, 0x017F, 1), // Old English long s (ſ), folds to S/s.
            (0x212A, 0x212A, 1), // Kelvin K, folds to K/k.
        ],
        r32: &[],
        latin_offset: 0,
    };

    // Go: regexp/syntax/parse.go:1657 aliases
    /// aliases is a lazily constructed copy of unicode.CategoryAliases and unicode.Scripts
    /// but with the keys passed through canonicalName, to support inexact matches.
    // PORT: a LazyLock replaces the struct's sync.Once.
    struct Aliases {
        categories: HashMap<String, &'static str>,
        scripts: HashMap<String, &'static str>,
    }


    // Go: regexp/syntax/parse.go:1664 initAliases

    // Go: regexp/syntax/parse.go:1680 canonicalName

    // Go: regexp/syntax/parse.go:1720 unicodeTable

    // Go: regexp/syntax/parse.go:1932 cleanClass

    // Go: regexp/syntax/parse.go:1979 appendLiteral
    /// appendLiteral appends the literal x to the class r.
    fn append_literal(r: &mut Vec<Rune>, x: Rune, flags: Flags) {
        if flags & FOLD_CASE != 0 {
            append_folded_range(r, x, x);
        } else {
            append_range(r, x, x);
        }
    }

    // Go: regexp/syntax/parse.go:1987 appendRange
    /// appendRange appends the range lo-hi to the class r.
    fn append_range(r: &mut Vec<Rune>, lo: Rune, hi: Rune) {
        // Expand last range or next to last range if it overlaps or abuts.
        // Checking two ranges helps when appending case-folded
        // alphabets, so that one range can be expanding A-Z and the
        // other expanding a-z.
        let n = r.len();
        for i in [2, 4] {
            // twice, using i=2, i=4
            if n >= i {
                let (rlo, rhi) = (r[n - i], r[n - i + 1]);
                if lo <= rhi + 1 && rlo <= hi + 1 {
                    if lo < rlo {
                        r[n - i] = lo;
                    }
                    if hi > rhi {
                        r[n - i + 1] = hi;
                    }
                    return;
                }
            }
        }

        r.push(lo);
        r.push(hi);
    }

    // minimum and maximum runes involved in folding.
    // checked during test.
    const MIN_FOLD: Rune = 0x0041;
    const MAX_FOLD: Rune = 0x1e943;

    // Go: regexp/syntax/parse.go:2020 appendFoldedRange
    /// appendFoldedRange appends the range lo-hi
    /// and its case folding-equivalent runes to the class r.
    fn append_folded_range(r: &mut Vec<Rune>, mut lo: Rune, mut hi: Rune) {
        // Optimizations.
        if lo <= MIN_FOLD && hi >= MAX_FOLD {
            // Range is full: folding can't add more.
            append_range(r, lo, hi);
            return;
        }
        if hi < MIN_FOLD || lo > MAX_FOLD {
            // Range is outside folding possibilities.
            append_range(r, lo, hi);
            return;
        }
        if lo < MIN_FOLD {
            // [lo, minFold-1] needs no folding.
            append_range(r, lo, MIN_FOLD - 1);
            lo = MIN_FOLD;
        }
        if hi > MAX_FOLD {
            // [maxFold+1, hi] needs no folding.
            append_range(r, MAX_FOLD + 1, hi);
            hi = MAX_FOLD;
        }

        // Brute force. Depend on appendRange to coalesce ranges on the fly.
        for c in lo..=hi {
            append_range(r, c, c);
            let mut f = unicode::simple_fold(c);
            while f != c {
                append_range(r, f, f);
                f = unicode::simple_fold(f);
            }
        }
    }

    // Go: regexp/syntax/parse.go:2055 appendClass

    // Go: regexp/syntax/parse.go:2063 appendFoldedClass

    // Go: regexp/syntax/parse.go:2072 appendNegatedClass

    /// The ranges of a RangeTable, R16 then R32, as runes.
    fn table_ranges(x: &RangeTable) -> impl Iterator<Item = (Rune, Rune, Rune)> + '_ {
        let r16 = x
            .r16
            .iter()
            .map(|&(lo, hi, stride)| (Rune::from(lo), Rune::from(hi), Rune::from(stride)));
        let r32 = x
            .r32
            .iter()
            .map(|&(lo, hi, stride)| (lo as Rune, hi as Rune, stride as Rune));
        r16.chain(r32)
    }

    // Go: regexp/syntax/parse.go:2088 appendTable
    /// appendTable appends x to the class r.
    fn append_table(r: &mut Vec<Rune>, x: &RangeTable) {
        for (lo, hi, stride) in table_ranges(x) {
            if stride == 1 {
                append_range(r, lo, hi);
                continue;
            }
            let mut c = lo;
            while c <= hi {
                append_range(r, c, c);
                c += stride;
            }
        }
    }

    // Go: regexp/syntax/parse.go:2113 appendNegatedTable
    /// appendNegatedTable appends the negation of x to the class r.
    fn append_negated_table(r: &mut Vec<Rune>, x: &RangeTable) {
        let mut next_lo: Rune = 0; // lo end of next class to add
        for (lo, hi, stride) in table_ranges(x) {
            if stride == 1 {
                if next_lo <= lo - 1 {
                    append_range(r, next_lo, lo - 1);
                }
                next_lo = hi + 1;
                continue;
            }
            let mut c = lo;
            while c <= hi {
                if next_lo <= c - 1 {
                    append_range(r, next_lo, c - 1);
                }
                next_lo = c + 1;
                c += stride;
            }
        }
        if next_lo <= MAX_RUNE {
            append_range(r, next_lo, MAX_RUNE);
        }
    }

    // Go: regexp/syntax/parse.go:2155 negateClass
    /// negateClass overwrites r with its negation.
    /// It assumes the class r is already clean.
    fn negate_class(r: &mut Vec<Rune>) {
        let mut next_lo: Rune = 0; // lo end of next class to add
        let mut w = 0; // write index
        let mut i = 0;
        while i < r.len() {
            let (lo, hi) = (r[i], r[i + 1]);
            i += 2;
            if next_lo <= lo - 1 {
                r[w] = next_lo;
                r[w + 1] = lo - 1;
                w += 2;
            }
            next_lo = hi + 1;
        }
        r.truncate(w);
        if next_lo <= MAX_RUNE {
            // It's possible for the negation to have one more
            // range - this one - than the original class, so use append.
            r.push(next_lo);
            r.push(MAX_RUNE);
        }
    }

    // Go: regexp/syntax/parse.go:2213 nextRune
    /// nextRune returns the first rune of s and the rest of s.
    // PORT: s is valid UTF-8, so there is no ErrInvalidUTF8. For an empty s,
    // Go's utf8.DecodeRuneInString gives RuneError and an empty rest.
    fn next_rune(s: &str) -> (char, &str) {
        match s.chars().next() {
            Some(c) => (c, &s[c.len_utf8()..]),
            None => ('\u{FFFD}', s),
        }
    }

    // Go: regexp/syntax/parse.go:2221 isalnum
    fn isalnum(c: char) -> bool {
        c.is_ascii_alphanumeric()
    }

    // Go: regexp/syntax/parse.go:2225 unhex
    fn unhex(c: char) -> Rune {
        match c {
            '0'..='9' => c as Rune - '0' as Rune,
            'a'..='f' => c as Rune - 'a' as Rune + 10,
            'A'..='F' => c as Rune - 'A' as Rune + 10,
            _ => -1,
        }
    }

    // Go: regexp/syntax/simplify.go:14 Simplify

    /// Adds a node to the arena and returns its index (Go `&Regexp{...}`).
    fn new_node(nodes: &mut Vec<Regexp>, re: Regexp) -> usize {
        nodes.push(re);
        nodes.len() - 1
    }

    // Go: regexp/syntax/simplify.go:134 simplify1
    /// simplify1 implements Simplify for the unary OpStar,
    /// OpPlus, and OpQuest operators. It returns the simple regexp
    /// equivalent to
    ///
    /// ```text
    /// Regexp{Op: op, Flags: flags, Sub: {sub}}
    /// ```
    ///
    /// under the assumption that sub is already simple, and
    /// without first allocating that structure. If the regexp
    /// to be returned turns out to be equivalent to re, simplify1
    /// returns re instead.
    ///
    /// simplify1 is factored out of Simplify because the implementation
    /// for other operators generates these unary expressions.
    /// Letting them call simplify1 makes sure the expressions they
    /// generate are simple.
    fn simplify1(
        nodes: &mut Vec<Regexp>,
        op: Op,
        flags: Flags,
        sub: usize,
        re: Option<usize>,
    ) -> usize {
        // Special case: repeat the empty string as much as
        // you want, but it's still the empty string.
        if nodes[sub].op == OP_EMPTY_MATCH {
            return sub;
        }
        // The operators are idempotent if the flags match.
        if op == nodes[sub].op && flags & NON_GREEDY == nodes[sub].flags & NON_GREEDY {
            return sub;
        }
        if let Some(re) = re {
            if nodes[re].op == op
                && nodes[re].flags & NON_GREEDY == flags & NON_GREEDY
                && sub == nodes[re].sub[0]
            {
                return re;
            }
        }

        new_node(
            nodes,
            Regexp {
                op,
                flags,
                sub: vec![sub],
                ..Regexp::default()
            },
        )
    }

    // Go: regexp/syntax/prog.go:18 Prog
    /// A Prog is a compiled regular expression program.
    pub struct Prog {
        pub inst: Vec<Inst>,
        /// index of start instruction
        pub start: usize,
        /// number of InstCapture insts in re
        pub num_cap: usize,
    }

    // Go: regexp/syntax/prog.go:25 InstOp
    /// An InstOp is an instruction opcode.
    pub type InstOp = u8;

    pub const INST_ALT: InstOp = 0;
    pub const INST_ALT_MATCH: InstOp = 1;
    pub const INST_CAPTURE: InstOp = 2;
    pub const INST_EMPTY_WIDTH: InstOp = 3;
    pub const INST_MATCH: InstOp = 4;
    pub const INST_FAIL: InstOp = 5;
    pub const INST_NOP: InstOp = 6;
    pub const INST_RUNE: InstOp = 7;
    pub const INST_RUNE1: InstOp = 8;
    pub const INST_RUNE_ANY: InstOp = 9;
    pub const INST_RUNE_ANY_NOT_NL: InstOp = 10;

    // Go: regexp/syntax/prog.go:63 EmptyOp
    /// An EmptyOp specifies a kind or mixture of zero-width assertions.
    pub type EmptyOp = u8;

    pub const EMPTY_BEGIN_LINE: EmptyOp = 1 << 0;
    pub const EMPTY_END_LINE: EmptyOp = 1 << 1;
    pub const EMPTY_BEGIN_TEXT: EmptyOp = 1 << 2;
    pub const EMPTY_END_TEXT: EmptyOp = 1 << 3;
    pub const EMPTY_WORD_BOUNDARY: EmptyOp = 1 << 4;
    pub const EMPTY_NO_WORD_BOUNDARY: EmptyOp = 1 << 5;

    // Go: regexp/syntax/prog.go:108 IsWordChar
    /// IsWordChar reports whether r is considered a “word character”
    /// during the evaluation of the \b and \B zero-width assertions.
    /// These assertions are ASCII-only: the word characters are [A-Za-z0-9_].
    pub fn is_word_char(r: Rune) -> bool {
        // Test for lowercase letters first, as these occur more
        // frequently than uppercase letters in common cases.
        ('a' as Rune..='z' as Rune).contains(&r)
            || ('A' as Rune..='Z' as Rune).contains(&r)
            || ('0' as Rune..='9' as Rune).contains(&r)
            || r == '_' as Rune
    }

    // Go: regexp/syntax/prog.go:114 Inst
    /// An Inst is a single instruction in a regular expression program.
    #[derive(Clone, Debug, Default)]
    pub struct Inst {
        pub op: InstOp,
        /// all but InstMatch, InstFail
        pub out: u32,
        /// InstAlt, InstAltMatch, InstCapture, InstEmptyWidth
        pub arg: u32,
        pub rune: Vec<Rune>,
    }

    impl Prog {
        // Go: regexp/syntax/prog.go:129 skipNop
        /// skipNop follows any no-op or capturing instructions.
        fn skip_nop(&self, pc: u32) -> &Inst {
            let mut i = &self.inst[pc as usize];
            while i.op == INST_NOP || i.op == INST_CAPTURE {
                i = &self.inst[i.out as usize];
            }
            i
        }

        // Go: regexp/syntax/prog.go:150 Prefix
        /// Prefix returns a literal string that all matches for the
        /// regexp must start with. Complete is true if the prefix
        /// is the entire match.
        pub fn prefix(&self) -> (String, bool) {
            let mut i = self.skip_nop(self.start as u32);

            // Avoid allocation of buffer if prefix is empty.
            if i.op_() != INST_RUNE || i.rune.len() != 1 {
                return (String::new(), i.op == INST_MATCH);
            }

            // Have prefix; gather characters.
            let mut buf = String::new();
            while i.op_() == INST_RUNE
                && i.rune.len() == 1
                && (i.arg as Flags) & FOLD_CASE == 0
                && i.rune[0] != super::UTF8_RUNE_ERROR
            {
                // PORT: Go's WriteRune writes RuneError for a surrogate or
                // an out-of-range rune.
                buf.push(char::from_u32(i.rune[0] as u32).unwrap_or('\u{FFFD}'));
                i = self.skip_nop(i.out);
            }
            (buf, i.op == INST_MATCH)
        }

        // Go: regexp/syntax/prog.go:169 StartCond
        /// StartCond returns the leading empty-width conditions that must
        /// be true in any match. It returns ^EmptyOp(0) if no matches are possible.
        pub fn start_cond(&self) -> EmptyOp {
            let mut flag: EmptyOp = 0;
            let mut pc = self.start;
            let mut i = &self.inst[pc];
            loop {
                match i.op {
                    INST_EMPTY_WIDTH => flag |= i.arg as EmptyOp,
                    INST_FAIL => return !0,
                    INST_CAPTURE | INST_NOP => {
                        // skip
                    }
                    _ => break,
                }
                pc = i.out as usize;
                i = &self.inst[pc];
            }
            flag
        }
    }

    const NO_MATCH: i32 = -1;

    impl Inst {
        // Go: regexp/syntax/prog.go:138 op
        /// op returns i.Op but merges all the Rune special cases into InstRune
        fn op_(&self) -> InstOp {
            match self.op {
                INST_RUNE1 | INST_RUNE_ANY | INST_RUNE_ANY_NOT_NL => INST_RUNE,
                op => op,
            }
        }

        // Go: regexp/syntax/prog.go:195 MatchRune
        /// MatchRune reports whether the instruction matches (and consumes) r.
        /// It should only be called when i.Op == [InstRune].
        pub fn match_rune(&self, r: Rune) -> bool {
            self.match_rune_pos(r) != NO_MATCH
        }

        // Go: regexp/syntax/prog.go:204 MatchRunePos
        /// MatchRunePos checks whether the instruction matches (and consumes) r.
        /// If so, MatchRunePos returns the index of the matching rune pair
        /// (or, when len(i.Rune) == 1, rune singleton).
        /// If not, MatchRunePos returns -1.
        /// MatchRunePos should only be called when i.Op == [InstRune].
        pub fn match_rune_pos(&self, r: Rune) -> i32 {
            let rune = &self.rune;

            match rune.len() {
                0 => return NO_MATCH,

                1 => {
                    // Special case: single-rune slice is from literal string, not char class.
                    let r0 = rune[0];
                    if r == r0 {
                        return 0;
                    }
                    if (self.arg as Flags) & FOLD_CASE != 0 {
                        let mut r1 = unicode::simple_fold(r0);
                        while r1 != r0 {
                            if r == r1 {
                                return 0;
                            }
                            r1 = unicode::simple_fold(r1);
                        }
                    }
                    return NO_MATCH;
                }

                2 => {
                    if r >= rune[0] && r <= rune[1] {
                        return 0;
                    }
                    return NO_MATCH;
                }

                4 | 6 | 8 => {
                    // Linear search for a few pairs.
                    // Should handle ASCII well.
                    let mut j = 0;
                    while j < rune.len() {
                        if r < rune[j] {
                            return NO_MATCH;
                        }
                        if r <= rune[j + 1] {
                            return (j / 2) as i32;
                        }
                        j += 2;
                    }
                    return NO_MATCH;
                }

                _ => {}
            }

            // Otherwise binary search.
            let mut lo = 0;
            let mut hi = rune.len() / 2;
            while lo < hi {
                let m = (lo + hi) >> 1;
                let c = rune[2 * m];
                if c <= r {
                    if r <= rune[2 * m + 1] {
                        return m as i32;
                    }
                    lo = m + 1;
                } else {
                    hi = m;
                }
            }
            NO_MATCH
        }
    }

    // Go: regexp/syntax/compile.go:20 patchList
    /// A patchList is a list of instruction pointers that need to be filled in (patched).
    /// Because the pointers haven't been filled in yet, we can reuse their storage
    /// to hold the list. It's kind of sleazy, but works well in practice.
    /// See https://swtch.com/~rsc/regexp/regexp1.html for inspiration.
    ///
    /// These aren't really pointers: they're integers, so we can reinterpret them
    /// this way without using package unsafe. A value l.head denotes
    /// p.inst[l.head>>1].Out (l.head&1==0) or .Arg (l.head&1==1).
    /// head == 0 denotes the empty list, okay because we start every program
    /// with a fail instruction, so we'll never want to point at its output link.
    #[derive(Clone, Copy, Default)]
    struct PatchList {
        head: u32,
        tail: u32,
    }

    // Go: regexp/syntax/compile.go:23 makePatchList
    fn make_patch_list(n: u32) -> PatchList {
        PatchList { head: n, tail: n }
    }

    impl PatchList {
        // Go: regexp/syntax/compile.go:27 patch
        fn patch(self, p: &mut Prog, val: u32) {
            let mut head = self.head;
            while head != 0 {
                let i = &mut p.inst[(head >> 1) as usize];
                if head & 1 == 0 {
                    head = i.out;
                    i.out = val;
                } else {
                    head = i.arg;
                    i.arg = val;
                }
            }
        }

        // Go: regexp/syntax/compile.go:41 append
        fn append(self, p: &mut Prog, l2: PatchList) -> PatchList {
            if self.head == 0 {
                return l2;
            }
            if l2.head == 0 {
                return self;
            }

            let i = &mut p.inst[(self.tail >> 1) as usize];
            if self.tail & 1 == 0 {
                i.out = l2.head;
            } else {
                i.arg = l2.head;
            }
            PatchList {
                head: self.head,
                tail: l2.tail,
            }
        }
    }

    // Go: regexp/syntax/compile.go:58 frag
    /// A frag represents a compiled program fragment.
    #[derive(Clone, Copy, Default)]
    struct Frag {
        /// index of first instruction
        i: u32,
        /// where to record end instruction
        out: PatchList,
        /// whether fragment can match empty string
        nullable: bool,
    }

    // Go: regexp/syntax/compile.go:64 compiler
    struct Compiler<'a> {
        p: Prog,
        nodes: &'a [Regexp],
    }

    // Go: regexp/syntax/compile.go:71 Compile
    /// Compile compiles the regexp into a program to be executed.
    /// The regexp should have been simplified already (returned from re.Simplify).
    // PORT: Go also returns an error, which is always nil.
    pub fn compile(nodes: &[Regexp], re: usize) -> Prog {
        let mut c = Compiler::init(nodes);
        let f = c.compile(re);
        let m = c.inst(INST_MATCH).i;
        f.out.patch(&mut c.p, m);
        c.p.start = f.i as usize;
        c.p
    }

    /// anyRuneNotNL and anyRune.
    const ANY_RUNE_NOT_NL: [Rune; 4] = [0, '\n' as Rune - 1, '\n' as Rune + 1, MAX_RUNE];
    const ANY_RUNE: [Rune; 2] = [0, MAX_RUNE];

    impl<'a> Compiler<'a> {
        // Go: regexp/syntax/compile.go:80 init
        fn init(nodes: &'a [Regexp]) -> Self {
            let mut c = Compiler {
                p: Prog {
                    inst: Vec::new(),
                    start: 0,
                    num_cap: 2, // implicit ( and ) for whole match $0
                },
                nodes,
            };
            c.inst(INST_FAIL);
            c
        }

        // Go: regexp/syntax/compile.go:89 compile
        fn compile(&mut self, re: usize) -> Frag {
            let nodes = self.nodes;
            let node = &nodes[re];
            match node.op {
                OP_NO_MATCH => self.fail(),
                OP_EMPTY_MATCH => self.nop(),
                OP_LITERAL => {
                    if node.rune.is_empty() {
                        return self.nop();
                    }
                    let mut f = Frag::default();
                    for j in 0..node.rune.len() {
                        let f1 = self.rune(&node.rune[j..=j], node.flags);
                        if j == 0 {
                            f = f1;
                        } else {
                            f = self.cat(f, f1);
                        }
                    }
                    f
                }
                OP_CHAR_CLASS => self.rune(&node.rune, node.flags),
                OP_ANY_CHAR_NOT_NL => self.rune(&ANY_RUNE_NOT_NL, 0),
                OP_ANY_CHAR => self.rune(&ANY_RUNE, 0),
                OP_BEGIN_LINE => self.empty(EMPTY_BEGIN_LINE),
                OP_END_LINE => self.empty(EMPTY_END_LINE),
                OP_BEGIN_TEXT => self.empty(EMPTY_BEGIN_TEXT),
                OP_END_TEXT => self.empty(EMPTY_END_TEXT),
                OP_WORD_BOUNDARY => self.empty(EMPTY_WORD_BOUNDARY),
                OP_NO_WORD_BOUNDARY => self.empty(EMPTY_NO_WORD_BOUNDARY),
                OP_CAPTURE => {
                    let bra = self.cap((node.cap << 1) as u32);
                    let sub = self.compile(node.sub[0]);
                    let ket = self.cap((node.cap << 1 | 1) as u32);
                    let f = self.cat(bra, sub);
                    self.cat(f, ket)
                }
                OP_STAR => {
                    let f = self.compile(node.sub[0]);
                    self.star(f, node.flags & NON_GREEDY != 0)
                }
                OP_PLUS => {
                    let f = self.compile(node.sub[0]);
                    self.plus(f, node.flags & NON_GREEDY != 0)
                }
                OP_QUEST => {
                    let f = self.compile(node.sub[0]);
                    self.quest(f, node.flags & NON_GREEDY != 0)
                }
                OP_CONCAT => {
                    if node.sub.is_empty() {
                        return self.nop();
                    }
                    let mut f = Frag::default();
                    for (i, &sub) in node.sub.iter().enumerate() {
                        if i == 0 {
                            f = self.compile(sub);
                        } else {
                            let f2 = self.compile(sub);
                            f = self.cat(f, f2);
                        }
                    }
                    f
                }
                OP_ALTERNATE => {
                    let mut f = Frag::default();
                    for &sub in &node.sub {
                        let f2 = self.compile(sub);
                        f = self.alt(f, f2);
                    }
                    f
                }
                _ => panic!("regexp: unhandled case in compile"),
            }
        }

        // Go: regexp/syntax/compile.go:161 inst
        fn inst(&mut self, op: InstOp) -> Frag {
            // TODO: impose length limit
            let f = Frag {
                i: self.p.inst.len() as u32,
                nullable: true,
                ..Frag::default()
            };
            self.p.inst.push(Inst {
                op,
                ..Inst::default()
            });
            f
        }

        // Go: regexp/syntax/compile.go:168 nop
        fn nop(&mut self) -> Frag {
            let mut f = self.inst(INST_NOP);
            f.out = make_patch_list(f.i << 1);
            f
        }

        // Go: regexp/syntax/compile.go:174 fail
        fn fail(&mut self) -> Frag {
            Frag::default()
        }

        // Go: regexp/syntax/compile.go:178 cap
        fn cap(&mut self, arg: u32) -> Frag {
            let mut f = self.inst(INST_CAPTURE);
            f.out = make_patch_list(f.i << 1);
            self.p.inst[f.i as usize].arg = arg;

            if self.p.num_cap < arg as usize + 1 {
                self.p.num_cap = arg as usize + 1;
            }
            f
        }

        // Go: regexp/syntax/compile.go:189 cat
        fn cat(&mut self, f1: Frag, f2: Frag) -> Frag {
            // concat of failure is failure
            if f1.i == 0 || f2.i == 0 {
                return Frag::default();
            }

            // TODO: elide nop

            f1.out.patch(&mut self.p, f2.i);
            Frag {
                i: f1.i,
                out: f2.out,
                nullable: f1.nullable && f2.nullable,
            }
        }

        // Go: regexp/syntax/compile.go:201 alt
        fn alt(&mut self, f1: Frag, f2: Frag) -> Frag {
            // alt of failure is other
            if f1.i == 0 {
                return f2;
            }
            if f2.i == 0 {
                return f1;
            }

            let mut f = self.inst(INST_ALT);
            let i = &mut self.p.inst[f.i as usize];
            i.out = f1.i;
            i.arg = f2.i;
            f.out = f1.out.append(&mut self.p, f2.out);
            f.nullable = f1.nullable || f2.nullable;
            f
        }

        // Go: regexp/syntax/compile.go:219 quest
        fn quest(&mut self, f1: Frag, nongreedy: bool) -> Frag {
            let mut f = self.inst(INST_ALT);
            let i = &mut self.p.inst[f.i as usize];
            if nongreedy {
                i.arg = f1.i;
                f.out = make_patch_list(f.i << 1);
            } else {
                i.out = f1.i;
                f.out = make_patch_list(f.i << 1 | 1);
            }
            f.out = f.out.append(&mut self.p, f1.out);
            f
        }

        // Go: regexp/syntax/compile.go:238 loop
        /// loop returns the fragment for the main loop of a plus or star.
        /// For plus, it can be used after changing the entry to f1.i.
        /// For star, it can be used directly when f1 can't match an empty string.
        /// (When f1 can match an empty string, f1* must be implemented as (f1+)?
        /// to get the priority match order correct.)
        fn loop_(&mut self, f1: Frag, nongreedy: bool) -> Frag {
            let mut f = self.inst(INST_ALT);
            let i = &mut self.p.inst[f.i as usize];
            if nongreedy {
                i.arg = f1.i;
                f.out = make_patch_list(f.i << 1);
            } else {
                i.out = f1.i;
                f.out = make_patch_list(f.i << 1 | 1);
            }
            f1.out.patch(&mut self.p, f.i);
            f
        }

        // Go: regexp/syntax/compile.go:252 star
        fn star(&mut self, f1: Frag, nongreedy: bool) -> Frag {
            if f1.nullable {
                // Use (f1+)? to get priority match order correct.
                // See golang.org/issue/46123.
                let f = self.plus(f1, nongreedy);
                return self.quest(f, nongreedy);
            }
            self.loop_(f1, nongreedy)
        }

        // Go: regexp/syntax/compile.go:261 plus
        fn plus(&mut self, f1: Frag, nongreedy: bool) -> Frag {
            Frag {
                i: f1.i,
                out: self.loop_(f1, nongreedy).out,
                nullable: f1.nullable,
            }
        }

        // Go: regexp/syntax/compile.go:265 empty
        fn empty(&mut self, op: EmptyOp) -> Frag {
            let mut f = self.inst(INST_EMPTY_WIDTH);
            self.p.inst[f.i as usize].arg = u32::from(op);
            f.out = make_patch_list(f.i << 1);
            f
        }

        // Go: regexp/syntax/compile.go:272 rune
        fn rune(&mut self, r: &[Rune], flags: Flags) -> Frag {
            let mut f = self.inst(INST_RUNE);
            f.nullable = false;
            let i = &mut self.p.inst[f.i as usize];
            i.rune = r.to_vec();
            let mut flags = flags & FOLD_CASE; // only relevant flag is FoldCase
            if r.len() != 1 || unicode::simple_fold(r[0]) == r[0] {
                // and sometimes not even that
                flags &= !FOLD_CASE;
            }
            i.arg = u32::from(flags);
            f.out = make_patch_list(f.i << 1);

            // Special cases for exec machine.
            if flags & FOLD_CASE == 0 && (r.len() == 1 || r.len() == 2 && r[0] == r[1]) {
                i.op = INST_RUNE1;
            } else if r.len() == 2 && r[0] == 0 && r[1] == MAX_RUNE {
                i.op = INST_RUNE_ANY;
            } else if r.len() == 4
                && r[0] == 0
                && r[1] == '\n' as Rune - 1
                && r[2] == '\n' as Rune + 1
                && r[3] == MAX_RUNE
            {
                i.op = INST_RUNE_ANY_NOT_NL;
            }

            f
        }
    }
}

/// Go `unicode`: the simple case folding that `regexp/syntax` uses, over
/// the generated tables of `unicode_tables` (`CaseRanges`, `caseOrbit` and
/// `asciiFold`).
mod unicode {
    use super::syntax::{MAX_RUNE, Rune};
    use crate::gostd::unicode_tables::{ASCII_FOLD, CASE_ORBIT, CASE_RANGES, CaseRange};

    const UPPER_CASE: usize = 0;
    const LOWER_CASE: usize = 1;

    /// Go `unicode.MaxASCII`.
    const MAX_ASCII: Rune = 0x7F;

    // Go: unicode/letter.go:354 SimpleFold
    /// SimpleFold iterates over Unicode code points equivalent under
    /// the Unicode-defined simple case folding. Among the code points
    /// equivalent to rune (including rune itself), SimpleFold returns the
    /// smallest rune > r if one exists, or else the smallest rune >= 0.
    /// If r is not a valid Unicode code point, SimpleFold(r) returns r.
    pub fn simple_fold(r: Rune) -> Rune {
        if !(0..=MAX_RUNE).contains(&r) {
            return r;
        }

        if r <= MAX_ASCII {
            return Rune::from(ASCII_FOLD[r as usize]);
        }

        // Consult caseOrbit table for special cases.
        let mut lo = 0;
        let mut hi = CASE_ORBIT.len();
        while lo < hi {
            let m = (lo + hi) >> 1;
            if Rune::from(CASE_ORBIT[m].0) < r {
                lo = m + 1;
            } else {
                hi = m;
            }
        }
        if lo < CASE_ORBIT.len() && Rune::from(CASE_ORBIT[lo].0) == r {
            return Rune::from(CASE_ORBIT[lo].1);
        }

        // No folding specified. This is a one- or two-element
        // equivalence class containing rune and ToLower(rune)
        // and ToUpper(rune) if they are different from rune.
        if let Some(cr) = lookup_case_range(r, CASE_RANGES) {
            let l = convert_case(LOWER_CASE, r, cr);
            if l != r {
                return l;
            }
            return convert_case(UPPER_CASE, r, cr);
        }
        r
    }

    // Go: unicode/letter.go:211 lookupCaseRange
    /// lookupCaseRange returns the CaseRange mapping for rune r or None if no
    /// mapping exists for r.
    fn lookup_case_range(r: Rune, case_range: &[CaseRange]) -> Option<&CaseRange> {
        // binary search over ranges
        let mut lo = 0;
        let mut hi = case_range.len();
        while lo < hi {
            let m = (lo + hi) >> 1;
            let cr = &case_range[m];
            if cr.0 as Rune <= r && r <= cr.1 as Rune {
                return Some(cr);
            }
            if r < cr.0 as Rune {
                hi = m;
            } else {
                lo = m + 1;
            }
        }
        None
    }

    // Go: unicode/letter.go:231 convertCase
    /// convertCase converts r to _case using CaseRange cr.
    fn convert_case(case: usize, r: Rune, cr: &CaseRange) -> Rune {
        let delta = cr.2[case];
        if delta > MAX_RUNE {
            // In an Upper-Lower sequence, which always starts with
            // an UpperCase letter, the real deltas always look like:
            //	{0, 1, 0}    UpperCase (Lower is next)
            //	{-1, 0, -1}  LowerCase (Upper, Title are previous)
            // The characters at even offsets from the beginning of the
            // sequence are upper case; the ones at odd offsets are lower.
            // The correct mapping can be done by clearing or setting the low
            // bit in the sequence offset.
            // The constants UpperCase and TitleCase are even while LowerCase
            // is odd so we take the low bit from _case.
            let lo = cr.0 as Rune;
            return lo + ((r - lo) & !1 | (case & 1) as Rune);
        }
        r + delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // go1.27.1 `unicodeTable` (regexp/syntax/parse.go:1720) also finds a
    // script by its canonical name, so `\p{Old_Italic}` compiles. go1.26.8
    // said "invalid character class range". Garay is a Unicode 16.0 script.
    #[test]
    fn script_names_match_inexactly() {
        for p in [r"\p{Old_Italic}", r"\p{olditalic}"] {
            let re = compile_exported(p).expect(p);
            assert!(re.match_string("\u{10300}"), "{p}");
            assert!(!re.match_string("\u{264}"), "{p}");
        }
        let re = compile_exported(r"\p{Garay}").expect("Garay");
        assert!(re.match_string("\u{10D40}"));
    }

    // go1.27.1 SimpleFold (Unicode 17.0.0) folds U+A7CB and U+0264.
    #[test]
    fn case_folding_uses_unicode_17() {
        let re = compile_exported(r"(?i)\x{A7CB}").expect("compile");
        assert!(re.match_string("\u{264}"));
    }
}

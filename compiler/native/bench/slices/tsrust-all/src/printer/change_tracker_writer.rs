//! Port of Go `printer/changetrackerwriter.go`.

use crate::prelude::*;

use crate::frontend::scanner::scanner_p1::{};

// Go: printer/changetrackerwriter.go:12 ChangeTrackerWriter
// PORT: Go embeds `textWriter`; here it is the field `text_writer`, and the
// promoted methods are the `EmitTextWriter` impl below. The print handlers
// (closures the printer calls while it also writes to this writer) and the
// writer share `lastNonTriviaPosition`, `pos` and `end` through `positions`.
// Borrows of `positions` are short and never nested.
pub struct ChangeTrackerWriter {
    text_writer: TextWriter,
    positions: Rc<RefCell<TriviaPositions>>,
}

/// The Go `ChangeTrackerWriter` fields that the print handlers share.
// PORT: not a Go type; see `ChangeTrackerWriter`.
#[derive(Debug, Default)]
struct TriviaPositions {
    last_non_trivia_position: i32,
    pos: FxHashMap<TriviaPositionKey, i32>,
    end: FxHashMap<TriviaPositionKey, i32>,
}

// Go: printer/changetrackerwriter.go:19 triviaPositionKey
// Go: interface { // *astNode | *ast.NodeList
// PORT: Go keys the maps by pointer. `NodeList` has no `Hash`, so a list is
// keyed by the address of the list it names (`NodeList::list_ptr`; pointer
// identity, as Go and `NodeList`'s `PartialEq`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TriviaPositionKey {
    Node(Node),
    NodeList(usize),
}

impl TriviaPositionKey {
}

impl TriviaPositions {
    // Go: printer/changetrackerwriter.go:74 setPos
    fn set_pos(&mut self, node: TriviaPositionKey) {
        self.pos.insert(node, self.last_non_trivia_position);
    }

    // Go: printer/changetrackerwriter.go:78 setEnd
    fn set_end(&mut self, node: TriviaPositionKey) {
        self.end.insert(node, self.last_non_trivia_position);
    }

    // Go: printer/changetrackerwriter.go:82 getPos
    fn get_pos(&self, node: TriviaPositionKey) -> i32 {
        self.pos.get(&node).copied().unwrap_or(0)
    }

    // Go: printer/changetrackerwriter.go:86 getEnd
    fn get_end(&self, node: TriviaPositionKey) -> i32 {
        self.end.get(&node).copied().unwrap_or(0)
    }
}

// Go: printer/changetrackerwriter.go:24 NewChangeTrackerWriter

impl ChangeTrackerWriter {
    // Go: printer/changetrackerwriter.go:39 GetPrintHandlers
    // PORT: the handlers hold the shared `positions` instead of `ct`.

    // Go: printer/changetrackerwriter.go:74 setPos
    fn set_pos(&self, node: TriviaPositionKey) {
        self.positions.borrow_mut().set_pos(node);
    }

    // Go: printer/changetrackerwriter.go:78 setEnd
    fn set_end(&self, node: TriviaPositionKey) {
        self.positions.borrow_mut().set_end(node);
    }

    // Go: printer/changetrackerwriter.go:82 getPos
    fn get_pos(&self, node: TriviaPositionKey) -> i32 {
        self.positions.borrow().get_pos(node)
    }

    // Go: printer/changetrackerwriter.go:86 getEnd
    fn get_end(&self, node: TriviaPositionKey) -> i32 {
        self.positions.borrow().get_end(node)
    }

    // Go: printer/changetrackerwriter.go:90 setLastNonTriviaPosition

    // Go: printer/changetrackerwriter.go:107 AssignPositionsToNode

    // Go: printer/changetrackerwriter.go:130 assignPositionsToNodeWorker

    // Go: printer/changetrackerwriter.go:154 assignPositionsToNodeArray
}


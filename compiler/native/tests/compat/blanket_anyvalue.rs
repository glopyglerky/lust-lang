// K1 compatibility fixture: one blanket impl (AnyValue shape with
// dyn Any downcast).
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/frontend/json_ext.rs AnyValue
// (verbatim below, with MarshalerTo renamed to the local trait
// Describe so the fixture stands alone; in ts-rust the supertrait is
// the JSON MarshalerTo of the json kernel). The blanket impl makes
// every T: Any + Describe + Debug + Send an AnyValue, including a
// Box<dyn AnyValue> itself, which is the documented trap this
// fixture records.

use std::any::Any;
use std::fmt::Debug;

/// Stand-in supertrait (MarshalerTo in ts-rust): one method is enough
/// to keep the object-safety and blanket-impl shape.
trait Describe {
    fn describe(&self, out: &mut String);
}

/// Go any holding a value that the LSP layer can marshal. Read it
/// with downcast_ref::<T>().
///
/// Never box a Box<dyn AnyValue> again: the blanket impl makes the
/// box itself an AnyValue, and a downcast of the outer box fails.
trait AnyValue: Any + Describe + Debug + Send {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Describe + Debug + Send> AnyValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn AnyValue {
    /// Go type assertion v.(T) for a non-nil any.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

// Box delegation, as in ts-rust json_ext.rs (impl<T: MarshalerTo +
// ?Sized> MarshalerTo for Box<T>). This impl is what makes a
// Box<dyn AnyValue> itself satisfy the blanket impl above.
impl<T: Describe + ?Sized> Describe for Box<T> {
    fn describe(&self, out: &mut String) {
        (**self).describe(out)
    }
}

#[derive(Debug)]
struct Position {
    line: u32,
    character: u32,
}

impl Describe for Position {
    fn describe(&self, out: &mut String) {
        out.push_str(&format!("{}:{}", self.line, self.character));
    }
}

#[derive(Debug)]
struct Range {
    start: Position,
    end: Position,
}

impl Describe for Range {
    fn describe(&self, out: &mut String) {
        self.start.describe(out);
        out.push('-');
        self.end.describe(out);
    }
}

// The blanket impl also covers plain values like strings once they
// implement the supertrait.
impl Describe for String {
    fn describe(&self, out: &mut String) {
        out.push_str(self);
    }
}

fn show(label: &str, value: &dyn AnyValue) {
    let mut text = String::new();
    value.describe(&mut text);
    println!("{label}: describe={text} debug={value:?}");
    println!(
        "{label}: as Position={:?} as Range={:?} as String={:?}",
        value.downcast_ref::<Position>(),
        value.downcast_ref::<Range>().map(|r| (r.start.line, r.end.line)),
        value.downcast_ref::<String>(),
    );
}

fn main() {
    let pos: Box<dyn AnyValue> = Box::new(Position {
        line: 3,
        character: 17,
    });
    let range: Box<dyn AnyValue> = Box::new(Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 5,
        },
    });
    let text: Box<dyn AnyValue> = Box::new("hello".to_string());
    show("pos", &*pos);
    show("range", &*range);
    show("text", &*text);

    // The documented trap: boxing the box makes the outer box the
    // value, so the inner type no longer downcasts.
    let boxed_box: Box<dyn AnyValue> = Box::new(pos);
    println!(
        "boxed box: as Position={:?} as Box={:?}",
        boxed_box.downcast_ref::<Position>(),
        boxed_box.downcast_ref::<Box<dyn AnyValue>>().map(|_| "hit"),
    );

    // A heterogeneous list, as in LSP params.
    let mut params: Vec<Box<dyn AnyValue>> = Vec::new();
    params.push(range);
    params.push(text);
    for (i, p) in params.iter().enumerate() {
        let mut text = String::new();
        p.describe(&mut text);
        println!("param {i}: {text}");
    }
}

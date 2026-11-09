//! Port of Go `transformers/estransforms/esdecorator.go` lines 1 to 1720:
//! the transformer state, its constructor, the lexical scope stack, the root
//! visitors, `transformClassLike` and the class element visitors. The rest is
//! in `es_decorator_p2.rs`.
//!
//! PORT: Go keeps twelve `*ast.NodeVisitor` fields. They are built on demand
//! with `TxVisitors::with_visitor`. Go shares `*classInfo` between the scope
//! stack and `classInfoStack` and mutates it in place; it is
//! `Rc<RefCell<ClassInfo>>`. The scope stack is a `Box` linked list.

// Class/Decorator evaluation order, as it pertains to this transformer:
//
// 1. Class decorators are evaluated outside of the private name scope of the class.
//    - 15.8.20 RS: BindingClassDeclarationEvaluation
//    - 15.8.21 RS: Evaluation
//    - 8.3.5 RS: NamedEvaluation
// 2. ClassHeritage clause is evaluated outside of the private name scope of the class.
//    - 15.8.19 RS: ClassDefinitionEvaluation, Step 8.c.
// 3. The name of the class is assigned.
// 4. For each member:
//    a. Member Decorators are evaluated.
//       - 15.8.19 RS: ClassDefinitionEvaluation, Step 23.
//       - Probably 15.7.13 RS: ClassElementEvaluation, but it's missing from spec text.
//    b. Computed Property name is evaluated
//       - 15.8.19 RS: ClassDefinitionEvaluation, Step 23.
//       - 15.8.15 RS: ClassFieldDefinitionEvaluation, Step 1.
//       - 15.4.5 RS: MethodDefinitionEvaluation, Step 1.
// 5. Static non-field (method/getter/setter/auto-accessor) element decorators are applied
// 6. Non-static non-field (method/getter/setter/auto-accessor) element decorators are applied
// 7. Static field (excl. auto-accessor) element decorators are applied
// 8. Non-static field (excl. auto-accessor) element decorators are applied
// 9. Class decorators are applied
// 10. Class binding is initialized
// 11. Static method extra initializers are evaluated
// 12. Static fields are initialized (incl. extra initializers) and static blocks are evaluated
// 13. Class extra initializers are evaluated
//
// Class constructor evaluation order, as it pertains to this transformer:
//
// 1. Instance method extra initializers are evaluated
// 2. For each instance field/auto-accessor:
//    a. The field is initialized and defined on the instance.
//    b. Extra initializers for the field are evaluated.

use super::contract::{};
use super::named_evaluation::{
    
    
    
};
use crate::prelude::*;
use crate::printer::factory::AssignedNameOptions;
use crate::printer::{AutoGenerateOptions, EmitFlags, GeneratedIdentifierFlags};
use crate::transformers::utilities::{
    
    
};

// Go: transformers/estransforms/esdecorator.go:48 lexicalEntryKind
/// lexicalEntryKind discriminates the kind of lexical scope entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LexicalEntryKind {
    Class,
    ClassElement,
    Name,
    Other,
}

// Go: transformers/estransforms/esdecorator.go:59 lexicalEntry
/// lexicalEntry represents a single entry in the lexical scope stack used to track
/// nested class declarations and their state during transformation.
pub(super) struct LexicalEntry {
    pub(super) kind: LexicalEntryKind,
    pub(super) next: Option<Box<LexicalEntry>>,
    pub(super) class_info_data: Option<ClassInfoRef>,
    pub(super) saved_pending_expressions: Vec<Node>,
    pub(super) class_this_data: Node,
    pub(super) class_super_data: Node,
    pub(super) depth: i32,
}

impl LexicalEntry {
    fn new(kind: LexicalEntryKind, next: Option<Box<LexicalEntry>>) -> Self {
        Self {
            kind,
            next,
            class_info_data: None,
            saved_pending_expressions: Vec::new(),
            class_this_data: Node::NIL,
            class_super_data: Node::NIL,
            depth: 0,
        }
    }
}

// Go: transformers/estransforms/esdecorator.go:70 memberInfo
/// memberInfo stores decoration-related data for a single class element.
#[derive(Clone, Copy, Default)]
pub(super) struct MemberInfo {
    pub(super) member_decorators_name: Node, // used in class definition step 4.a
    pub(super) member_initializers_name: Node, // used in class definition step 12 and constructor evaluation step 2.a
    pub(super) member_extra_initializers_name: Node, // used in class definition step 12 and constructor evaluation step 2.b
    pub(super) member_descriptor_name: Node,
}

// Go: transformers/estransforms/esdecorator.go:78 classInfo
/// classInfo stores all transformation data for a single decorated class.
#[derive(Default)]
pub(super) struct ClassInfo {
    pub(super) class: Node,
    pub(super) class_decorators_name: Node, // used in class definition step 2
    pub(super) class_descriptor_name: Node, // used in class definition step 10
    pub(super) class_extra_initializers_name: Node, // used in class definition step 13
    pub(super) class_this: Node,            // `_classThis`, if needed.
    pub(super) class_super: Node,           // `_classSuper`, if needed.
    pub(super) metadata_reference: Node,
    pub(super) member_infos: IndexMap<Node, MemberInfo>, // used in class definition step 4.a, 12, and constructor evaluation
    pub(super) instance_method_extra_initializers_name: Node, // used in constructor evaluation step 1
    pub(super) static_method_extra_initializers_name: Node,   // used in class definition step 11
    pub(super) static_non_field_decoration_statements: Vec<Node>,
    pub(super) non_static_non_field_decoration_statements: Vec<Node>,
    pub(super) static_field_decoration_statements: Vec<Node>,
    pub(super) non_static_field_decoration_statements: Vec<Node>,
    pub(super) has_static_initializers: bool,
    pub(super) has_non_ambient_instance_fields: bool,
    pub(super) has_static_private_class_elements: bool,
    pub(super) pending_static_initializers: Vec<Node>,
    pub(super) pending_instance_initializers: Vec<Node>,
}

/// Go `*classInfo`.
pub(super) type ClassInfoRef = Rc<RefCell<ClassInfo>>;

// Go: transformers/estransforms/esdecorator.go:100 esDecoratorTransformer


// Go: transformers/estransforms/esdecorator.go:124 newESDecoratorTransformer


// Go: transformers/estransforms/esdecorator.go:1232 partialResult


/// Go `printer.AutoGenerateOptions{Flags: GeneratedIdentifierFlagsOptimistic | GeneratedIdentifierFlagsFileLevel}`.
pub(super) fn optimistic_file_level() -> AutoGenerateOptions {
    AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
        ..Default::default()
    }
}


// Go: transformers/estransforms/esdecorator.go:431 getHelperVariableName

// Go: transformers/estransforms/esdecorator.go:1014 isDecoratedClassLike

// Go: transformers/estransforms/esdecorator.go:1877 isAnonymousClassNeedingAssignedName

// Go: transformers/estransforms/esdecorator.go:1885 canIgnoreEmptyStringLiteralInAssignedName

// Go: transformers/estransforms/esdecorator.go:2719 injectClassThisAssignmentIfMissing

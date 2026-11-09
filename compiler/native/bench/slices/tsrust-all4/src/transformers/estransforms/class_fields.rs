//! Port of Go `transformers/estransforms/classfields.go` lines 1 to 1580:
//! the transformer state, its constructor and the visitors up to
//! `visitParenthesizedExpression`. The rest is in `class_fields_p2.rs`.
//!
//! PORT: Go keeps nine `*ast.NodeVisitor` fields. They are built on demand
//! with `TxVisitors::with_visitor` and the Go callback (for example
//! `tx.discardedValueVisitor.VisitNode(n)` is
//! `self.with_visitor(Self::visit_discarded_value, |v| v.visit_node(n))`).
//! Go shares the class lexical environment and private environment records
//! through pointers and mutates them in place; they are `Rc<RefCell<..>>`.

use super::contract::{TransformReferenceResolver, };
use super::named_evaluation::{
    
};
use crate::prelude::*;
use crate::printer::factory::PrivateIdentifierKind;
use crate::printer::{AutoGenerateOptions, EmitFlags, GeneratedIdentifierFlags};
use crate::transformers::utilities::{
    get_non_assignment_operator_for_compound_assignment, 
};

// Go: transformers/estransforms/classfields.go:18 classFacts
/// classFacts tracks various facts about a class being transformed.
pub(super) type ClassFacts = u32;

pub(super) const CLASS_FACTS_NONE: ClassFacts = 0;
pub(super) const CLASS_FACTS_CLASS_WAS_DECORATED: ClassFacts = 1 << 0;
pub(super) const CLASS_FACTS_NEEDS_CLASS_CONSTRUCTOR_REFERENCE: ClassFacts = 1 << 1;
pub(super) const CLASS_FACTS_NEEDS_CLASS_SUPER_REFERENCE: ClassFacts = 1 << 2;
pub(super) const CLASS_FACTS_NEEDS_SUBSTITUTION_FOR_THIS_IN_CLASS_STATIC_FIELD: ClassFacts = 1 << 3;
pub(super) const CLASS_FACTS_WILL_HOIST_INITIALIZERS_TO_CONSTRUCTOR: ClassFacts = 1 << 4;


// Go: transformers/estransforms/classfields.go:31 privateIdentifierInfo
/// privateIdentifierInfo stores information about a private identifier during transformation.
#[derive(Clone, Debug)]
pub(super) struct PrivateIdentifierInfo {
    pub(super) kind: PrivateIdentifierKind,
    /// brandCheckIdentifier can contain:
    ///  - For instance field: The WeakMap that will be the storage for the field.
    ///  - For instance methods or accessors: The WeakSet that will be used for brand checking.
    ///  - For static members: The constructor that will be used for brand checking.
    pub(super) brand_check_identifier: Node,
    /// isStatic stores if the identifier is static or not.
    pub(super) is_static: bool,
    /// isValid stores if the identifier declaration is valid or not. Reserved names (e.g. #constructor)
    /// or duplicate identifiers are considered invalid.
    pub(super) is_valid: bool,
    /// variableName contains the variable that will serve as the storage for a static field.
    pub(super) variable_name: Node,
    /// methodName is the identifier for a variable that will contain the private method implementation.
    pub(super) method_name: Node,
    /// getterName is the identifier for a variable that will contain the private get accessor implementation, if any.
    pub(super) getter_name: Node,
    /// setterName is the identifier for a variable that will contain the private set accessor implementation, if any.
    pub(super) setter_name: Node,
}

impl PrivateIdentifierInfo {
    /// Go `&privateIdentifierInfo{kind: kind}` (all other fields zero).
    pub(super) fn new(kind: PrivateIdentifierKind) -> Self {
        Self {
            kind,
            brand_check_identifier: Node::NIL,
            is_static: false,
            is_valid: false,
            variable_name: Node::NIL,
            method_name: Node::NIL,
            getter_name: Node::NIL,
            setter_name: Node::NIL,
        }
    }
}

/// Go `*privateIdentifierInfo`.
pub(super) type PrivateIdentifierInfoRef = Rc<RefCell<PrivateIdentifierInfo>>;

// Go: transformers/estransforms/classfields.go:54 privateEnvironmentData
/// privateEnvironmentData stores class-scoped environment data for private identifiers.
#[derive(Clone, Copy, Default)]
pub(super) struct PrivateEnvironmentData {
    /// className is used for prefixing generated variable names.
    pub(super) class_name: Node,
    /// weakSetName is used for brand check on private methods.
    pub(super) weak_set_name: Node,
}

// Go: transformers/estransforms/classfields.go:65 privateEnvironment
/// privateEnvironment stores a map of private identifier names to their transform info.
/// Like Strada, it uses two separate maps: one for non-generated identifiers (keyed by text)
/// and one for generated identifiers (keyed by original AST node). This prevents collisions
/// when different auto-accessors produce generated backing field names with the same text.
#[derive(Default)]
pub(super) struct PrivateEnvironment {
    pub(super) data: PrivateEnvironmentData,
    pub(super) members: FxHashMap<String, PrivateIdentifierInfoRef>,
    pub(super) generated_identifiers: FxHashMap<Node, PrivateIdentifierInfoRef>,
}

/// Go `*privateEnvironment`.
pub(super) type PrivateEnvironmentRef = Rc<RefCell<PrivateEnvironment>>;

// Go: transformers/estransforms/classfields.go:72 classLexicalEnvironment
/// classLexicalEnvironment stores information about the lexical environment of a class.
#[derive(Clone, Copy, Default)]
pub(super) struct ClassLexicalEnvironment {
    pub(super) facts: ClassFacts,
    /// classConstructor is used for brand checks on static members, and `this` references in static initializers.
    pub(super) class_constructor: Node,
    pub(super) class_this: Node,
    /// superClassReference is used for `super` references in static initializers.
    pub(super) super_class_reference: Node,
}

/// Go `*classLexicalEnvironment`.
pub(super) type ClassLexicalEnvironmentRef = Rc<RefCell<ClassLexicalEnvironment>>;

// Go: transformers/estransforms/classfields.go:82 classLexicalEnv
/// classLexicalEnv is a linked list of class lexical environments.
#[derive(Default)]
pub(super) struct ClassLexicalEnv {
    pub(super) previous: Option<Rc<ClassLexicalEnv>>,
    pub(super) data: RefCell<Option<ClassLexicalEnvironmentRef>>,
    pub(super) private_env: RefCell<Option<PrivateEnvironmentRef>>,
}

impl ClassLexicalEnv {
    /// Go `env.data` (nil is `None`).
    pub(super) fn data(&self) -> Option<ClassLexicalEnvironmentRef> {
        self.data.borrow().clone()
    }

    /// Go `env.privateEnv` (nil is `None`).
    pub(super) fn private_env(&self) -> Option<PrivateEnvironmentRef> {
        self.private_env.borrow().clone()
    }
}

// Go: transformers/estransforms/classfields.go:88 classFieldsTransformer


// Go: transformers/estransforms/classfields.go:140 newClassFieldsTransformer


// Go: transformers/estransforms/classfields.go:3395 isStaticPropertyDeclarationOrClassStaticBlock

// Go: transformers/estransforms/classfields.go:3547 flattenCommaList

// Go: transformers/estransforms/classfields.go:3553 flattenCommaListWorker

// Go: transformers/estransforms/classfields.go:3564 findComputedPropertyNameCacheAssignment

// Go: transformers/estransforms/classfields.go:3580 expandPreOrPostfixIncrementOrDecrementExpression

/// Go `printer.AutoGenerateOptions{Flags: printer.GeneratedIdentifierFlagsReservedInNestedScopes}`.
pub(super) fn reserved_in_nested_scopes() -> AutoGenerateOptions {
    AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
        ..Default::default()
    }
}

// ARCH 4.4 and 9.3 W-CORELANG: the nested ts-rust Type / TypeData
// / TypeReference / TypeMapper field-type graph, rebuilt as a
// standalone program with local stand-in types of equal layout.
// Field order, repr, and variant order follow checker/types.rs
// and checker/mapper.rs in ts-rust (read only). No ts-rust code
// is imported, so this case needs no source std or lifetime
// support beyond plain references. The same shape re-runs end
// to end at the first stage that supports all of its parts.
//
// Stand-ins, each chosen for equal size and alignment:
// - Handles (TypeId, SymbolId, ...) are u32 newtypes, Node is a
//   u64 newtype, and flag types are u32 newtypes, as in core.rs
//   (handle!) and flags.rs (go_flags! with u32).
// - FlatMap keeps its real fields (checker/flat_map.rs:32) and
//   CacheHashKey keeps #[repr(C, packed(4))] (checker_p19.rs).
//   Note: packed(4) is outside the ARCH 4.4 supported-shape
//   grammar, which allows packed with no N or N = 1 only. This
//   corpus records the real shape; the grammar gap is reported.
// - FxHashMap uses a zero-sized local hasher, like rustc-hash.
//   Only boxed payloads contain one, so it cannot change the
//   TypeData size either way.
// - Deferred and function mapper callbacks are Rc<dyn Fn>
//   values with local signatures; a fat pointer is a fat pointer.
#![allow(dead_code)]
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::mem::{align_of, offset_of, size_of, ManuallyDrop};
use std::rc::Rc;

// Handles and flags.
pub struct TypeId(pub u32);
pub struct SymbolId(pub u32);
pub struct SignatureId(pub u32);
pub struct IndexInfoId(pub u32);
pub struct MapperId(pub u32);
pub struct InferenceContextId(pub u32);
pub struct SymbolTable(pub u32);
pub struct InstantiationMapId(pub u32);
pub struct Name(u32);
pub struct Node(pub u64);
pub struct TypeFlags(pub u32);
pub struct ObjectFlags(pub u32);
pub struct ElementFlags(pub u32);
pub struct IndexFlags(pub u32);
pub struct AccessFlags(pub u32);
pub struct Number(pub f64);
pub struct PseudoBigInt {
    pub negative: bool,
    pub base10_value: String,
}

#[derive(Default)]
pub struct FxHasher;
impl Hasher for FxHasher {
    fn finish(&self) -> u64 {
        0
    }
    fn write(&mut self, _bytes: &[u8]) {}
}
pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[repr(C, packed(4))]
pub struct CacheHashKey {
    pub hi: u64,
    pub lo: u64,
}
pub struct FlatMap<K, V> {
    slots: Box<[(K, V)]>,
    used: usize,
    zero: Option<V>,
}
pub type InstantiationMap = FlatMap<CacheHashKey, TypeId>;

// SharedList (checker/types.rs:39): 24 bytes for the id lists.
pub struct SharedList<T: 'static> {
    repr: ManuallyDrop<SharedListRepr<T>>,
}
enum SharedListRepr<T: 'static> {
    Arena(&'static [T]),
    Owned(Rc<[T]>),
    Inline { len: InlineLen, items: [T; 3] },
}
#[repr(u8)]
enum InlineLen {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

// ArenaBox (checker/types.rs:1442): one non-null pointer word.
pub struct ArenaBox<T: 'static>(ArenaBoxRepr<T>);
enum ArenaBoxRepr<T: 'static> {
    Arena(&'static mut T),
    Owned(Box<T>),
}

pub struct TypeAlias {
    pub symbol: SymbolId,
    pub type_arguments: Vec<TypeId>,
}

#[repr(C, align(64))]
pub struct Type {
    pub flags: TypeFlags,
    pub object_flags: ObjectFlags,
    pub id: TypeId,
    pub symbol: SymbolId,
    pub alias: Option<Rc<TypeAlias>>,
    pub data: TypeData,
}

pub enum TypeData {
    Intrinsic(IntrinsicType),
    Literal(LiteralType),
    UniqueESSymbol(UniqueESSymbolType),
    TypeParameter(TypeParameter),
    Index(IndexType),
    IndexedAccess(IndexedAccessType),
    TemplateLiteral(TemplateLiteralType),
    StringMapping(StringMappingType),
    Substitution(SubstitutionType),
    Conditional(ConditionalType),
    Object(ObjectType),
    TypeReference(TypeReference),
    Interface(Box<InterfaceType>),
    Tuple(Box<TupleType>),
    InstantiationExpression(Box<InstantiationExpressionType>),
    Mapped(Box<MappedType>),
    ReverseMapped(Box<ReverseMappedType>),
    EvolvingArray(Box<EvolvingArrayType>),
    Union(ArenaBox<UnionType>),
    Intersection(ArenaBox<IntersectionType>),
}

pub struct IntrinsicType {
    pub intrinsic_name: String,
}
pub enum LiteralValue {
    String(String),
    Number(Number),
    Bool(bool),
    PseudoBigInt(PseudoBigInt),
}
pub struct LiteralType {
    pub value: Option<LiteralValue>,
    pub fresh_type: TypeId,
    pub regular_type: TypeId,
    pub property_name: OnceCell<Name>,
    pub go_plain: OnceCell<bool>,
}
pub struct UniqueESSymbolType {
    pub name: String,
}
pub struct ConstrainedType {
    pub resolved_base_constraint: TypeId,
}
#[repr(C, align(64))]
pub struct StructuredSignatures {
    pub signatures: SharedList<SignatureId>,
    pub call_signature_count: i32,
    pub object_type_without_abstract_construct_signatures: TypeId,
    pub index_infos: SharedList<IndexInfoId>,
}
#[repr(C)]
pub struct StructuredType {
    pub constrained: ConstrainedType,
    pub members: SymbolTable,
    pub signatures_data: Option<ArenaBox<StructuredSignatures>>,
    pub properties: SharedList<SymbolId>,
}
#[repr(C)]
pub struct ObjectType {
    pub target: TypeId,
    pub mapper: MapperId,
    pub structured: StructuredType,
    pub instantiations: InstantiationMapId,
}
#[repr(u64)]
pub enum GenericArgumentsMemo {
    Unknown = 0,
    False = 1,
    True = 2,
}
#[repr(C)]
pub struct TypeReference {
    pub generic_arguments_memo: GenericArgumentsMemo,
    pub resolved_type_arguments: SharedList<TypeId>,
    pub object: ObjectType,
    pub node: Node,
}
pub struct TypeParameter {
    pub constrained: ConstrainedType,
    pub constraint: TypeId,
    pub target: TypeId,
    pub mapper: MapperId,
    pub is_this_type: bool,
    pub is_distributed: bool,
    pub resolved_default_type: TypeId,
    pub distributed_type: TypeId,
}
pub struct IndexType {
    pub constrained: ConstrainedType,
    pub target: TypeId,
    pub index_flags: IndexFlags,
}
pub struct IndexedAccessType {
    pub constrained: ConstrainedType,
    pub object_type: TypeId,
    pub index_type: TypeId,
    pub access_flags: AccessFlags,
}
pub struct TemplateLiteralType {
    pub constrained: ConstrainedType,
    pub texts: Rc<[String]>,
    pub types: SharedList<TypeId>,
    pub go_plain: bool,
}
pub struct StringMappingType {
    pub constrained: ConstrainedType,
    pub target: TypeId,
}
pub struct SubstitutionType {
    pub constrained: ConstrainedType,
    pub base_type: TypeId,
    pub constraint: TypeId,
}
pub struct ConditionalRoot {
    pub node: Node,
    pub check_type: TypeId,
    pub extends_type: TypeId,
    pub is_distributive: bool,
    pub infer_type_parameters: Vec<TypeId>,
    pub outer_type_parameters: SharedList<TypeId>,
    pub instantiations: Option<InstantiationMap>,
    pub alias: Option<Rc<TypeAlias>>,
    pub distribution_dependent: Option<bool>,
}
pub struct ConditionalType {
    pub constrained: ConstrainedType,
    pub root: Rc<RefCell<ConditionalRoot>>,
    pub check_type: TypeId,
    pub extends_type: TypeId,
    pub resolved_true_type: TypeId,
    pub resolved_false_type: TypeId,
    pub resolved_inferred_true_type: TypeId,
    pub resolved_default_constraint: TypeId,
    pub resolved_constraint_of_distributive: TypeId,
    pub mapper: MapperId,
    pub combined_mapper: MapperId,
}
pub struct InterfaceType {
    pub reference: TypeReference,
    pub all_type_parameters: Vec<TypeId>,
    pub outer_type_parameter_count: i32,
    pub this_type: TypeId,
    pub base_types_resolved: bool,
    pub declared_members_resolved: bool,
    pub resolved_base_constructor_type: TypeId,
    pub resolved_base_types: SharedList<TypeId>,
    pub declared_members: SymbolTable,
    pub declared_call_signatures: SharedList<SignatureId>,
    pub declared_construct_signatures: SharedList<SignatureId>,
    pub declared_index_infos: SharedList<IndexInfoId>,
    pub instantiations: Option<InstantiationMap>,
}
pub struct TupleElementInfo {
    pub flags: ElementFlags,
    pub labeled_declaration: Node,
}
pub struct TupleType {
    pub interface: InterfaceType,
    pub element_infos: Vec<TupleElementInfo>,
    pub min_length: i32,
    pub fixed_length: i32,
    pub combined_flags: ElementFlags,
    pub readonly: bool,
}
pub struct InstantiationExpressionType {
    pub object: ObjectType,
    pub node: Node,
}
pub struct MappedType {
    pub object: ObjectType,
    pub declaration: Node,
    pub type_parameter: TypeId,
    pub constraint_type: TypeId,
    pub name_type: TypeId,
    pub template_type: TypeId,
    pub modifiers_type: TypeId,
    pub resolved_apparent_type: TypeId,
    pub contains_error: bool,
}
pub struct ReverseMappedType {
    pub object: ObjectType,
    pub source: TypeId,
    pub mapped_type: TypeId,
    pub constraint_type: TypeId,
}
pub struct EvolvingArrayType {
    pub object: ObjectType,
    pub element_type: TypeId,
    pub final_array_type: TypeId,
}
pub struct UnionOrIntersectionType {
    pub structured: StructuredType,
    pub types: SharedList<TypeId>,
    pub property_cache: SymbolTable,
    pub property_cache_without_function_property_augment: SymbolTable,
    pub resolved_properties: SharedList<SymbolId>,
}
pub struct UnionType {
    pub union_or_intersection: UnionOrIntersectionType,
    pub resolved_reduced_type: TypeId,
    pub regular_type: TypeId,
    pub origin: TypeId,
    pub key_property_name: String,
    pub constituent_map: Option<FxHashMap<TypeId, TypeId>>,
}
pub struct IntersectionType {
    pub union_or_intersection: UnionOrIntersectionType,
    pub resolved_apparent_type: TypeId,
    pub unique_literal_filled_instantiation: TypeId,
}

// TypeMapper (checker/mapper.rs:40): asserted 16 bytes in ts-rust.
pub struct TypeMapperBase;
pub struct SimpleTypeMapper {
    pub source: TypeId,
    pub target: TypeId,
    pub maps_this_only: bool,
}
pub struct ArrayTypeMapper {
    pub sources: SharedList<TypeId>,
    pub targets: SharedList<TypeId>,
}
pub struct ArrayToSingleTypeMapper {
    pub sources: Vec<TypeId>,
    pub target: TypeId,
    pub maps_this_only: bool,
}
pub type DeferredTypeFn = Rc<dyn Fn(u32) -> u32>;
pub type TypeMapperFn = Rc<dyn Fn(u32, u32) -> u32>;
pub struct DeferredTypeMapper {
    pub sources: Vec<TypeId>,
    pub targets: Vec<DeferredTypeFn>,
    pub maps_this_only: bool,
}
pub struct FunctionTypeMapper {
    pub fn_: TypeMapperFn,
}
pub struct MergedTypeMapper {
    pub m1: MapperId,
    pub m2: MapperId,
}
pub struct CompositeTypeMapper {
    pub m1: MapperId,
    pub m2: MapperId,
}
pub struct InferenceTypeMapper {
    pub n: InferenceContextId,
    pub fixing: bool,
}
pub enum TypeMapper {
    Base(TypeMapperBase),
    Simple(SimpleTypeMapper),
    Array(Box<ArrayTypeMapper>, bool),
    ArrayToSingle(Box<ArrayToSingleTypeMapper>),
    Deferred(Box<DeferredTypeMapper>),
    Function(Box<FunctionTypeMapper>),
    Merged(MergedTypeMapper),
    Composite(CompositeTypeMapper),
    Inference(InferenceTypeMapper),
}

// The ts-rust const asserts (checker/types.rs:868-874 and the
// mapper asserts), restated on the stand-ins.
const _: () = assert!(size_of::<Type>() <= 128);
const _: () = assert!(offset_of!(Type, data) == 24);
const _: () = assert!(offset_of!(TypeReference, object.mapper) + 4 <= 40);
const _: () = assert!(size_of::<TypeData>() == size_of::<TypeReference>());
const _: () = assert!(size_of::<SharedList<TypeId>>() == 24);
const _: () = assert!(size_of::<StructuredSignatures>() == 64);
const _: () = assert!(size_of::<TypeMapper>() == 16);
const _: () = assert!(size_of::<ArrayTypeMapper>() == 48);

fn p<T>(name: &str) {
    println!("{name} size={} align={}", size_of::<T>(), align_of::<T>());
}
fn main() {
    p::<Type>("Type");
    p::<TypeData>("TypeData");
    p::<TypeReference>("TypeReference");
    p::<ObjectType>("ObjectType");
    p::<StructuredType>("StructuredType");
    p::<StructuredSignatures>("StructuredSignatures");
    p::<GenericArgumentsMemo>("GenericArgumentsMemo");
    p::<SharedList<TypeId>>("SharedList_TypeId");
    p::<SharedList<SymbolId>>("SharedList_SymbolId");
    p::<IntrinsicType>("IntrinsicType");
    p::<LiteralType>("LiteralType");
    p::<LiteralValue>("LiteralValue");
    p::<UniqueESSymbolType>("UniqueESSymbolType");
    p::<TypeParameter>("TypeParameter");
    p::<IndexType>("IndexType");
    p::<IndexedAccessType>("IndexedAccessType");
    p::<TemplateLiteralType>("TemplateLiteralType");
    p::<StringMappingType>("StringMappingType");
    p::<SubstitutionType>("SubstitutionType");
    p::<ConditionalType>("ConditionalType");
    p::<ConditionalRoot>("ConditionalRoot");
    p::<InterfaceType>("InterfaceType");
    p::<TupleType>("TupleType");
    p::<InstantiationExpressionType>("InstantiationExpressionType");
    p::<MappedType>("MappedType");
    p::<ReverseMappedType>("ReverseMappedType");
    p::<EvolvingArrayType>("EvolvingArrayType");
    p::<UnionType>("UnionType");
    p::<IntersectionType>("IntersectionType");
    p::<TypeMapper>("TypeMapper");
    p::<ArrayTypeMapper>("ArrayTypeMapper");
    p::<SimpleTypeMapper>("SimpleTypeMapper");
    println!("offset Type.data={}", offset_of!(Type, data));
    println!("offset TypeReference.object.mapper={}", offset_of!(TypeReference, object.mapper));
    println!("offset ObjectType.structured={}", offset_of!(ObjectType, structured));
}

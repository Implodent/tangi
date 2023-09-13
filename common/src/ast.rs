use derive_more::*;

use crate::interner::IdentifierID;

#[derive(Debug, Clone)]
pub struct File {
        pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
        Function(ItemFn),
        Constant(ItemConst),
}

#[derive(Debug, Clone)]
pub struct ItemFn {
        pub ident: IdentifierID,
        pub args: Vec<FnArgument>,
        pub return_type: Type,
        pub block: Block,
}

#[derive(Debug, Clone)]
pub struct ItemConst {
        pub ident: IdentifierID,
        pub ty: Type,
        pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct FnArgument {
        pub name: IdentifierID,
        pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
        Primitive(TypePrimitive),
        Reference(TypeReference),
        DynamicDispatch(TypeDyn),
        Nullable(Box<Type>),
        ErrorUnion(TypeErrorUnion),
        Path(Path),
}

impl Type {
        pub fn is_sized(&self) -> Option<bool> {
                match self {
                        Self::Primitive(primitive) => match primitive {
                                TypePrimitive::Array(TypeArray { size: None, .. })
                                | TypePrimitive::Str => Some(false),
                                TypePrimitive::Never => None,
                                _ => Some(true),
                        },
                        Self::Reference(_) => Some(true),
                        _ => None,
                }
        }
}

/// Represents a primitive type
#[derive(Debug, Clone)]
pub enum TypePrimitive {
        Number(TypeNumber),
        Array(TypeArray),
        Str,
        Never,
        Void,
        Bool,
}

#[derive(Debug, Clone)]
pub struct TypeDyn {
        pub tr: Path,
}

/// Represents a reference.
/// &'a mut T
#[derive(Debug, Clone)]
pub struct TypeReference {
        // 'a
        pub lifetime: Option<IdentifierID>,
        // mut
        pub mutable: bool,
        // T
        pub inner: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct TypeArray {
        pub inner: Box<Type>,
        pub size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TypeErrorUnion {
        pub error_type: Option<Box<Type>>,
        pub ok_type: Box<Type>,
}

/// Represents a number type, like u8 or i16
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeNumber {
        pub signed: Signedness,
        pub bits: u16,
}

/// Represents whether a number is signed or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Signedness {
        Signed = 1,
        Unsigned = 0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct Path(pub Vec<IdentifierID>);

#[derive(Debug, Clone)]
pub struct Block {
        pub label: Option<IdentifierID>,
        pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
        Call(ExprCall),
        Return(ExprReturn),
        If(ExprIf),
}

#[derive(Debug, Clone)]
pub struct ExprIf {
        pub condition: Box<Expression>,
        pub statements: Vec<Statement>,
        pub else_branches: Vec<ExprElse>,
}

#[derive(Debug, Clone)]
pub struct ExprElse {
        pub condition: Option<Box<Expression>>,
        pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ExprReturn {
        pub is_implicit: bool,
        pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ExprCall {
        pub callee: Path,
        pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
        Number(ExprNumber),
        Access(Path),
        Call(ExprCall),
        Return(ExprReturn),
        Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprNumber {
        Float(f64),
        Normal(ExprNumberNormal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprNumberNormal {
        pub radix: u32,
        pub number: i64,
        pub ty: Option<TypeNumber>,
}

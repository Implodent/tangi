use derive_more::*;

use crate::interner::IdentifierID;

#[derive(Debug, Clone)]
pub struct File {
        pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
        Function(ItemFn),
}

#[derive(Debug, Clone)]
pub struct ItemFn {
        pub ident: IdentifierID,
        pub args: Vec<FnArgument>,
        pub return_type: Type,
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
}

impl Type {
        pub fn is_sized(&self) -> Option<bool> {
                match self {
                        Self::Primitive(primitive) => match primitive {
                                TypePrimitive::Array(TypeArray {
                                        size: ArraySize::Unknown,
                                        ..
                                })
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
        pub size: ArraySize,
}

#[derive(Debug, Clone)]
pub struct TypeErrorUnion {
        pub error_type: Box<Option<Type>>,
        pub ok_type: Box<Type>,
}

#[derive(Debug, Clone)]
pub enum ArraySize {
        Known(usize),
        Unknown,
        ConstUnknown,
}

/// Represents a number type, like u8 or i16
#[derive(Debug, Clone, Copy)]
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

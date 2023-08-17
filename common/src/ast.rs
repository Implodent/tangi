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
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(TypePrimitive),
    Reference(TypeReference),
    DynamicDispatch(TypeDyn),
}

impl Type {
    pub fn is_sized(&self) -> Option<bool> {
        match self {
            Self::Primitive(primitive) => match primitive {
                TypePrimitive::Array(TypeArray { amount: None, .. }) | TypePrimitive::Str => {
                    Some(false)
                }
                TypePrimitive::Never => None,
                _ => Some(true),
            },
            Self::Reference(_) => Some(true),
            #[allow(unreachable_patterns)]
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
    pub lifetime: Option<Ident>,
    // mut
    pub mutable: bool,
    // T
    pub inner: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct TypeArray {
    pub inner: Box<Type>,
    pub amount: Option<usize>,
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

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Display)]
pub struct Path(pub Vec<Ident>);

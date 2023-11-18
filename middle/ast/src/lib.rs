use std::collections::HashMap;
pub type Ident = String;

pub struct Attribute {
    pub name: Ident,
    pub arguments: Vec<Expr>
}

pub struct File {
    pub items: Vec<Item>,
    pub attributes: Vec<Attribute>
}

pub enum Item {
    Struct(Structure),
    Enum(Enumeration),
    Fn(Function),
    Type(TypeAlias),
    Const(Constant),
    Static(Constant)
}

pub struct TypeAlias {
    pub vis: Visibility,
    pub name: Ident,
    pub arguments: Option<Vec<Type>>,
    pub actual: Type
}

pub struct Constant {
    pub vis: Visibility,
    pub name: Ident,
    pub value: Expr
}

pub enum Expr {
    
}

pub enum Visibility {
    Public,
    Inherited
}

pub enum Abi {
    C,
}

pub struct FunctionModifiers {
    pub const_: bool,
    pub async_: bool,
    // extern "ABI"
    pub abi: Option<Abi>
}

pub struct Function {
    pub vis: Visibility,
    pub modifiers: FunctionModifiers,
    pub name: Ident,
}

pub struct Enumeration {
    pub vis: Visibility,
    pub name: Ident,
    pub variants: Vec<EnumVariant>
}

pub struct EnumVariant {
    pub name: Ident,
    pub fields: EnumFields
}

pub enum EnumFields {
    Tuple(Vec<Type>),
    Struct(HashMap<Ident, Type>)
}

pub struct Structure {
    pub name: Ident,
    pub fields: Vec<StructField>
}

pub struct StructField {
    pub name: Ident,
    pub ty: Type
}

pub struct Type {
    pub kind: TypeKind,
    pub arguments: Option<Vec<Type>>
}

pub enum TypeKind {
    Primitive(TypePrimitive),
    Opaque(Ident)
}

pub enum TypePrimitive {
    Number(TypeNumber),
    Bool,
    Char,
    Str
}

pub enum TypeNumber {
    Int {
        signed: bool,
        bits: u8
    },
    Float(FloatBits)
}

pub enum FloatBits {
    F32,
    F64
}

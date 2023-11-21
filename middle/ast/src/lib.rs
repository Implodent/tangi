use std::collections::HashMap;
pub type Ident = String;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Ident,
    pub arguments: Vec<Expr>,
    pub inner: bool,
}

#[derive(Debug, Clone)]
pub struct File {
    pub items: Vec<Item>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Struct(Structure),
    Enum(Enumeration),
    Fn(Function),
    Type(TypeAlias),
    Const(Constant),
    Static(Constant),
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub vis: Visibility,
    pub name: Ident,
    pub arguments: Option<Vec<Type>>,
    pub actual: Type,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub vis: Visibility,
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Void,
    Opaque(Ident),
    Let(LetExpr),
    Primitive(PrimitiveExpr),
    Return(Box<Self>)
}

#[derive(Debug, Clone)]
pub enum PrimitiveExpr {
    Int { value: i64, bits: u8 },
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct LetExpr {
    pub mutable: bool,
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Void,
    // ref mut pat
    Ref(Box<Self>),
    Mut(Box<Self>),
    RefMut(Box<Self>),
    // var
    Variable(Ident),
    // var @ pat
    WithVariable(Ident, Box<Pattern>),
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Inherited,
}

#[derive(Debug, Clone, Copy)]
pub enum Abi {
    C,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FunctionModifiers {
    pub const_: bool,
    pub async_: bool,
    // extern "ABI"
    pub abi: Option<Abi>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub attributes: Vec<Attribute>,
    pub vis: Visibility,
    pub modifiers: FunctionModifiers,
    pub args: Vec<Type>,
    pub returns: Type,
    pub name: Ident,
    pub cap_args: Vec<Pattern>,
    pub statements: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Enumeration {
    pub vis: Visibility,
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub fields: EnumFields,
}

#[derive(Debug, Clone)]
pub enum EnumFields {
    Tuple(Vec<Type>),
    Struct(HashMap<Ident, Type>),
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: Ident,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub arguments: Option<Vec<Type>>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Primitive(TypePrimitive),
    Reference(Box<TypeReference>),
    Opaque(Ident),
}

#[derive(Debug, Clone)]
pub struct TypeReference {
    pub ty: Type,
    pub mutable: bool,
    pub lifetime: Option<Ident>,
}

#[derive(Debug, Clone)]
pub enum TypePrimitive {
    Void,
    Never,
    Number(TypeNumber),
    Bool,
    Char,
    Str,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeNumber {
    Int { signed: bool, bits: u8 },
    Float(FloatBits),
}

#[derive(Debug, Clone, Copy)]
pub enum FloatBits {
    F32,
    F64,
}

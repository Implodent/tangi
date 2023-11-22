use crate::index::IndexVec;

crate::define_index_type! {
    pub struct BasicBlockId = u32;
}

crate::define_index_type! {
    pub struct Local = u32;
}

crate::define_index_type! {
    pub struct FieldIdx = u32;
}

crate::define_index_type! {
    pub struct VariantIdx = u32;
}

pub struct Place {
    pub local: Local,
    pub projection: Vec<PlaceItem>
}

pub enum ProjectionElem<V, T> {
    Deref,
    Field(FieldIdx, T),
    Index(V),
    ConstantIndex {
        offset: u64,
        min_length: u64,
        from_end: bool,
    },
    Subslice {
        from: u64,
        to: u64,
        from_end: bool,
    },
    Downcast(Option<String>, VariantIdx),
    OpaqueCast(T),
    Subtype(T),
}

pub struct Type;
pub type PlaceItem = ProjectionElem<Local, Type>;

pub struct BasicBlocks {
    blocks: IndexVec<BasicBlockId, BasicBlock>,
}

impl BasicBlocks {
    pub fn new(blocks: IndexVec<BasicBlockId, BasicBlock>) -> Self {
        Self { blocks }
    }

    pub fn as_mut(&mut self) -> &mut IndexVec<BasicBlockId, BasicBlock> {
        &mut self.blocks
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: String,
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

#[derive(Debug, Clone)]
pub enum Statement {}

#[derive(Debug, Clone)]
pub enum Terminator {
    Goto { target: BasicBlockId },
    Return,
}

define_index_type! {
    pub struct DefIndex = u32;
}
define_index_type! {
    pub struct CrateNum = u32;
}

const CRATE_DEF_INDEX: CrateNum = CrateNum::from_raw_unchecked(0);

#[repr(C)]
pub struct DefId {
    pub krate: CrateNum,
    pub index: DefIndex,
}

impl DefId {
    pub fn local(index: DefIndex) -> Self {
        Self {
            krate: CRATE_DEF_INDEX,
            index,
        }
    }

    pub fn is_local(&self) -> bool {
        self.krate == CRATE_DEF_INDEX
    }
}

pub mod hir;
pub mod index;
pub mod mir;
pub mod ty;
pub mod util;

pub struct Cx {
    pub data_layout: DataLayout,
}
pub struct DataLayout {
    pub pointer_size: u32,
    pub pointer_align: u32,
}

#[macro_export]
macro_rules! explode {
    () => ( $crate::explode!("impossible case reached") );
    ($msg:expr) => ({ $crate::util::explode::bug_fmt(::std::format_args!($msg)) });
    ($msg:expr,) => ({ $crate::explode!($msg) });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::util::explode::explode_fmt(::std::format_args!($fmt, $($arg)+))
    });
}

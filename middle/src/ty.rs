use core::fmt;
use std::num::NonZeroU8;

use crate::{Cx, explode};

#[derive(Clone, Debug)]
pub enum Type {
    Xd
}

#[derive(Clone, Debug)]
pub enum Const {
    Param(ParamConst),
    Infer(InferConst),
    Value(ValTree)
}

pub enum ValTree {
    Leaf(ScalarInt),
    Branch(Vec<ValTree>)
}

/// The raw bytes of a simple value.
///
/// This is a packed struct in order to allow this type to be optimally embedded in enums
/// (like Scalar).
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(packed)]
pub struct ScalarInt {
    /// The first `size` bytes of `data` are the value.
    /// Do not try to read less or more bytes than that. The remaining bytes must be 0.
    data: u128,
    size: NonZeroU8,
}

impl ScalarInt {
    pub const TRUE: ScalarInt = ScalarInt { data: 1_u128, size: NonZeroU8::new(1).unwrap() };

    pub const FALSE: ScalarInt = ScalarInt { data: 0_u128, size: NonZeroU8::new(1).unwrap() };

    #[inline]
    pub fn size(self) -> Size {
        Size::from_bytes(self.size.get())
    }

    /// Make sure the `data` fits in `size`.
    /// This is guaranteed by all constructors here, but having had this check saved us from
    /// bugs many times in the past, so keeping it around is definitely worth it.
    #[inline(always)]
    fn check_data(self) {
        // Using a block `{self.data}` here to force a copy instead of using `self.data`
        // directly, because `debug_assert_eq` takes references to its arguments and formatting
        // arguments and would thus borrow `self.data`. Since `Self`
        // is a packed struct, that would create a possibly unaligned reference, which
        // is UB.
        debug_assert_eq!(
            self.size().truncate(self.data),
            { self.data },
            "Scalar value {:#x} exceeds size of {} bytes",
            { self.data },
            self.size
        );
    }

    #[inline]
    pub fn null(size: Size) -> Self {
        Self { data: 0, size: NonZeroU8::new(size.bytes() as u8).unwrap() }
    }

    #[inline]
    pub fn is_null(self) -> bool {
        self.data == 0
    }

    #[inline]
    pub fn try_from_uint(i: impl Into<u128>, size: Size) -> Option<Self> {
        let data = i.into();
        if size.truncate(data) == data {
            Some(Self { data, size: NonZeroU8::new(size.bytes() as u8).unwrap() })
        } else {
            None
        }
    }

    #[inline]
    pub fn try_from_int(i: impl Into<i128>, size: Size) -> Option<Self> {
        let i = i.into();
        // `into` performed sign extension, we have to truncate
        let truncated = size.truncate(i as u128);
        if size.sign_extend(truncated) as i128 == i {
            Some(Self { data: truncated, size: NonZeroU8::new(size.bytes() as u8).unwrap() })
        } else {
            None
        }
    }

    #[inline]
    pub fn try_from_target_usize(i: impl Into<u128>, cx: Cx) -> Option<Self> {
        Self::try_from_uint(i, cx.data_layout.pointer_size)
    }

    #[inline]
    pub fn assert_bits(self, target_size: Size) -> u128 {
        self.to_bits(target_size).unwrap_or_else(|size| {
            explode!("expected int of size {}, but got size {}", target_size.bytes(), size.bytes())
        })
    }

    #[inline]
    pub fn to_bits(self, target_size: Size) -> Result<u128, Size> {
        assert_ne!(target_size.bytes(), 0, "you should never look at the bits of a ZST");
        if target_size.bytes() == u64::from(self.size.get()) {
            self.check_data();
            Ok(self.data)
        } else {
            Err(self.size())
        }
    }

    #[inline]
    pub fn try_to_target_usize(&self, cx: Cx) -> Result<u64, Size> {
        Ok(self.to_bits(cx.data_layout.pointer_size)? as u64)
    }

    /// Tries to convert the `ScalarInt` to an unsigned integer of the given size.
    /// Fails if the size of the `ScalarInt` is not equal to `size` and returns the
    /// `ScalarInt`s size in that case.
    #[inline]
    pub fn try_to_uint(self, size: Size) -> Result<u128, Size> {
        self.to_bits(size)
    }

    // Tries to convert the `ScalarInt` to `bool`. Fails if the `size` of the `ScalarInt`
    // in not equal to `Size { raw: 1 }` or if the value is not 0 or 1 and returns the `size`
    // value of the `ScalarInt` in that case.
    #[inline]
    pub fn try_to_bool(self) -> Result<bool, Size> {
        match self.try_to_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(self.size()),
        }
    }

    // Tries to convert the `ScalarInt` to `u8`. Fails if the `size` of the `ScalarInt`
    // in not equal to `Size { raw: 1 }` and returns the `size` value of the `ScalarInt` in
    // that case.
    #[inline]
    pub fn try_to_u8(self) -> Result<u8, Size> {
        self.to_bits(Size::from_bits(8)).map(|v| u8::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to `u16`. Fails if the size of the `ScalarInt`
    /// in not equal to `Size { raw: 2 }` and returns the `size` value of the `ScalarInt` in
    /// that case.
    #[inline]
    pub fn try_to_u16(self) -> Result<u16, Size> {
        self.to_bits(Size::from_bits(16)).map(|v| u16::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to `u32`. Fails if the `size` of the `ScalarInt`
    /// in not equal to `Size { raw: 4 }` and returns the `size` value of the `ScalarInt` in
    /// that case.
    #[inline]
    pub fn try_to_u32(self) -> Result<u32, Size> {
        self.to_bits(Size::from_bits(32)).map(|v| u32::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to `u64`. Fails if the `size` of the `ScalarInt`
    /// in not equal to `Size { raw: 8 }` and returns the `size` value of the `ScalarInt` in
    /// that case.
    #[inline]
    pub fn try_to_u64(self) -> Result<u64, Size> {
        self.to_bits(Size::from_bits(64)).map(|v| u64::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to `u128`. Fails if the `size` of the `ScalarInt`
    /// in not equal to `Size { raw: 16 }` and returns the `size` value of the `ScalarInt` in
    /// that case.
    #[inline]
    pub fn try_to_u128(self) -> Result<u128, Size> {
        self.to_bits(Size::from_bits(128))
    }

    /// Tries to convert the `ScalarInt` to a signed integer of the given size.
    /// Fails if the size of the `ScalarInt` is not equal to `size` and returns the
    /// `ScalarInt`s size in that case.
    #[inline]
    pub fn try_to_int(self, size: Size) -> Result<i128, Size> {
        let b = self.to_bits(size)?;
        Ok(size.sign_extend(b) as i128)
    }

    /// Tries to convert the `ScalarInt` to i8.
    /// Fails if the size of the `ScalarInt` is not equal to `Size { raw: 1 }`
    /// and returns the `ScalarInt`s size in that case.
    pub fn try_to_i8(self) -> Result<i8, Size> {
        self.try_to_int(Size::from_bits(8)).map(|v| i8::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to i16.
    /// Fails if the size of the `ScalarInt` is not equal to `Size { raw: 2 }`
    /// and returns the `ScalarInt`s size in that case.
    pub fn try_to_i16(self) -> Result<i16, Size> {
        self.try_to_int(Size::from_bits(16)).map(|v| i16::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to i32.
    /// Fails if the size of the `ScalarInt` is not equal to `Size { raw: 4 }`
    /// and returns the `ScalarInt`s size in that case.
    pub fn try_to_i32(self) -> Result<i32, Size> {
        self.try_to_int(Size::from_bits(32)).map(|v| i32::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to i64.
    /// Fails if the size of the `ScalarInt` is not equal to `Size { raw: 8 }`
    /// and returns the `ScalarInt`s size in that case.
    pub fn try_to_i64(self) -> Result<i64, Size> {
        self.try_to_int(Size::from_bits(64)).map(|v| i64::try_from(v).unwrap())
    }

    /// Tries to convert the `ScalarInt` to i128.
    /// Fails if the size of the `ScalarInt` is not equal to `Size { raw: 16 }`
    /// and returns the `ScalarInt`s size in that case.
    pub fn try_to_i128(self) -> Result<i128, Size> {
        self.try_to_int(Size::from_bits(128))
    }
}

macro_rules! from {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for ScalarInt {
                #[inline]
                fn from(u: $ty) -> Self {
                    Self {
                        data: u128::from(u),
                        size: NonZeroU8::new(std::mem::size_of::<$ty>() as u8).unwrap(),
                    }
                }
            }
        )*
    }
}

macro_rules! try_from {
    ($($ty:ty),*) => {
        $(
            impl TryFrom<ScalarInt> for $ty {
                type Error = Size;
                #[inline]
                fn try_from(int: ScalarInt) -> Result<Self, Size> {
                    // The `unwrap` cannot fail because to_bits (if it succeeds)
                    // is guaranteed to return a value that fits into the size.
                    int.to_bits(Size::from_bytes(std::mem::size_of::<$ty>()))
                       .map(|u| u.try_into().unwrap())
                }
            }
        )*
    }
}

from!(u8, u16, u32, u64, u128, bool);
try_from!(u8, u16, u32, u64, u128);

impl TryFrom<ScalarInt> for bool {
    type Error = Size;
    #[inline]
    fn try_from(int: ScalarInt) -> Result<Self, Size> {
        int.to_bits(Size::from_bytes(1)).and_then(|u| match u {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Size::from_bytes(1)),
        })
    }
}

impl From<char> for ScalarInt {
    #[inline]
    fn from(c: char) -> Self {
        Self { data: c as u128, size: NonZeroU8::new(std::mem::size_of::<char>() as u8).unwrap() }
    }
}

/// Error returned when a conversion from ScalarInt to char fails.
#[derive(Debug)]
pub struct CharTryFromScalarInt;

impl TryFrom<ScalarInt> for char {
    type Error = CharTryFromScalarInt;

    #[inline]
    fn try_from(int: ScalarInt) -> Result<Self, Self::Error> {
        let Ok(bits) = int.to_bits(Size::from_bytes(std::mem::size_of::<char>())) else {
            return Err(CharTryFromScalarInt);
        };
        match char::from_u32(bits.try_into().unwrap()) {
            Some(c) => Ok(c),
            None => Err(CharTryFromScalarInt),
        }
    }
}

impl From<Single> for ScalarInt {
    #[inline]
    fn from(f: Single) -> Self {
        // We trust apfloat to give us properly truncated data.
        Self { data: f.to_bits(), size: NonZeroU8::new((Single::BITS / 8) as u8).unwrap() }
    }
}

impl TryFrom<ScalarInt> for Single {
    type Error = Size;
    #[inline]
    fn try_from(int: ScalarInt) -> Result<Self, Size> {
        int.to_bits(Size::from_bytes(4)).map(Self::from_bits)
    }
}

impl From<Double> for ScalarInt {
    #[inline]
    fn from(f: Double) -> Self {
        // We trust apfloat to give us properly truncated data.
        Self { data: f.to_bits(), size: NonZeroU8::new((Double::BITS / 8) as u8).unwrap() }
    }
}

impl TryFrom<ScalarInt> for Double {
    type Error = Size;
    #[inline]
    fn try_from(int: ScalarInt) -> Result<Self, Size> {
        int.to_bits(Size::from_bytes(8)).map(Self::from_bits)
    }
}

impl fmt::Debug for ScalarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Dispatch to LowerHex below.
        write!(f, "0x{self:x}")
    }
}

impl fmt::LowerHex for ScalarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.check_data();
        if f.alternate() {
            // Like regular ints, alternate flag adds leading `0x`.
            write!(f, "0x")?;
        }
        // Format as hex number wide enough to fit any value of the given `size`.
        // So data=20, size=1 will be "0x14", but with size=4 it'll be "0x00000014".
        // Using a block `{self.data}` here to force a copy instead of using `self.data`
        // directly, because `write!` takes references to its formatting arguments and
        // would thus borrow `self.data`. Since `Self`
        // is a packed struct, that would create a possibly unaligned reference, which
        // is UB.
        write!(f, "{:01$x}", { self.data }, self.size.get() as usize * 2)
    }
}

impl fmt::UpperHex for ScalarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.check_data();
        // Format as hex number wide enough to fit any value of the given `size`.
        // So data=20, size=1 will be "0x14", but with size=4 it'll be "0x00000014".
        // Using a block `{self.data}` here to force a copy instead of using `self.data`
        // directly, because `write!` takes references to its formatting arguments and
        // would thus borrow `self.data`. Since `Self`
        // is a packed struct, that would create a possibly unaligned reference, which
        // is UB.
        write!(f, "{:01$X}", { self.data }, self.size.get() as usize * 2)
    }
}

impl fmt::Display for ScalarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.check_data();
        write!(f, "{}", { self.data })
    }
}

/// Size of a type in bytes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size {
    raw: u64,
}

// This is debug-printed a lot in larger structs, don't waste too much space there
impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Size({} bytes)", self.bytes())
    }
}

impl Size {
    pub const ZERO: Size = Size { raw: 0 };

    /// Rounds `bits` up to the next-higher byte boundary, if `bits` is
    /// not a multiple of 8.
    pub fn from_bits(bits: impl TryInto<u64>) -> Size {
        let bits = bits.try_into().ok().unwrap();
        // Avoid potential overflow from `bits + 7`.
        Size { raw: bits / 8 + ((bits % 8) + 7) / 8 }
    }

    #[inline]
    pub fn from_bytes(bytes: impl TryInto<u64>) -> Size {
        let bytes: u64 = bytes.try_into().ok().unwrap();
        Size { raw: bytes }
    }

    #[inline]
    pub fn bytes(self) -> u64 {
        self.raw
    }

    #[inline]
    pub fn bytes_usize(self) -> usize {
        self.bytes().try_into().unwrap()
    }

    #[inline]
    pub fn bits(self) -> u64 {
        #[cold]
        fn overflow(bytes: u64) -> ! {
            panic!("Size::bits: {bytes} bytes in bits doesn't fit in u64")
        }

        self.bytes().checked_mul(8).unwrap_or_else(|| overflow(self.bytes()))
    }

    #[inline]
    pub fn bits_usize(self) -> usize {
        self.bits().try_into().unwrap()
    }

    #[inline]
    pub fn align_to(self, align: Align) -> Size {
        let mask = align.bytes() - 1;
        Size::from_bytes((self.bytes() + mask) & !mask)
    }

    #[inline]
    pub fn is_aligned(self, align: Align) -> bool {
        let mask = align.bytes() - 1;
        self.bytes() & mask == 0
    }

    #[inline]
    pub fn checked_add<C: HasDataLayout>(self, offset: Size, cx: &C) -> Option<Size> {
        let dl = cx.data_layout();

        let bytes = self.bytes().checked_add(offset.bytes())?;

        if bytes < dl.obj_size_bound() { Some(Size::from_bytes(bytes)) } else { None }
    }

    #[inline]
    pub fn checked_mul<C: HasDataLayout>(self, count: u64, cx: &C) -> Option<Size> {
        let dl = cx.data_layout();

        let bytes = self.bytes().checked_mul(count)?;
        if bytes < dl.obj_size_bound() { Some(Size::from_bytes(bytes)) } else { None }
    }

    /// Truncates `value` to `self` bits and then sign-extends it to 128 bits
    /// (i.e., if it is negative, fill with 1's on the left).
    #[inline]
    pub fn sign_extend(self, value: u128) -> u128 {
        let size = self.bits();
        if size == 0 {
            // Truncated until nothing is left.
            return 0;
        }
        // Sign-extend it.
        let shift = 128 - size;
        // Shift the unsigned value to the left, then shift back to the right as signed
        // (essentially fills with sign bit on the left).
        (((value << shift) as i128) >> shift) as u128
    }

    /// Truncates `value` to `self` bits.
    #[inline]
    pub fn truncate(self, value: u128) -> u128 {
        let size = self.bits();
        if size == 0 {
            // Truncated until nothing is left.
            return 0;
        }
        let shift = 128 - size;
        // Truncate (shift left to drop out leftover values, shift right to fill with zeroes).
        (value << shift) >> shift
    }

    #[inline]
    pub fn signed_int_min(&self) -> i128 {
        self.sign_extend(1_u128 << (self.bits() - 1)) as i128
    }

    #[inline]
    pub fn signed_int_max(&self) -> i128 {
        i128::MAX >> (128 - self.bits())
    }

    #[inline]
    pub fn unsigned_int_max(&self) -> u128 {
        u128::MAX >> (128 - self.bits())
    }
}

// Panicking addition, subtraction and multiplication for convenience.
// Avoid during layout computation, return `LayoutError` instead.

impl Add for Size {
    type Output = Size;
    #[inline]
    fn add(self, other: Size) -> Size {
        Size::from_bytes(self.bytes().checked_add(other.bytes()).unwrap_or_else(|| {
            panic!("Size::add: {} + {} doesn't fit in u64", self.bytes(), other.bytes())
        }))
    }
}

impl Sub for Size {
    type Output = Size;
    #[inline]
    fn sub(self, other: Size) -> Size {
        Size::from_bytes(self.bytes().checked_sub(other.bytes()).unwrap_or_else(|| {
            panic!("Size::sub: {} - {} would result in negative size", self.bytes(), other.bytes())
        }))
    }
}

impl Mul<Size> for u64 {
    type Output = Size;
    #[inline]
    fn mul(self, size: Size) -> Size {
        size * self
    }
}

impl Mul<u64> for Size {
    type Output = Size;
    #[inline]
    fn mul(self, count: u64) -> Size {
        match self.bytes().checked_mul(count) {
            Some(bytes) => Size::from_bytes(bytes),
            None => panic!("Size::mul: {} * {} doesn't fit in u64", self.bytes(), count),
        }
    }
}

impl AddAssign for Size {
    #[inline]
    fn add_assign(&mut self, other: Size) {
        *self = *self + other;
    }
}

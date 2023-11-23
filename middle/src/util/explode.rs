use core::fmt;
use std::panic;

struct Explode;

#[cold]
#[inline(never)]
#[track_caller]
pub fn explode_fmt(args: fmt::Arguments<'_>) -> ! {
    panic::panic_any(Explode)
}

#![feature(const_mut_refs)]
mod memory;
mod files;
mod input;
mod threads;
mod graphics;
mod sound;

pub use memory::*;
pub use files::*;
pub use input::*;
pub use threads::*;
pub use graphics::*;
pub use sound::*;

#[cfg(feature="gol-slow")]
#[macro_export]
macro_rules! gol_assert { ($e:expr) => { if !$e { panic!("assert!") } }; }

#[macro_export]
macro_rules! invalid_code_path { () => { gol_assert!(1 == 1); } }

#[cfg(not(feature="gol-slow"))]
#[macro_export]
macro_rules! gol_assert { ($e:expr) => { () }; }

pub const PI32: f32 = 3.14159265359;

pub fn safe_truncate_u64(value: u64) -> u32 {
    gol_assert!(value <= 0xFFFFFFFF);
    return value as u32;
}

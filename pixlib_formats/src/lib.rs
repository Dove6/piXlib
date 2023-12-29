pub mod compression_algorithms;
pub mod file_formats;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(usize::BITS >= u32::BITS);

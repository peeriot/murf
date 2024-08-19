#![warn(
    clippy::pedantic,
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused
)]
#![doc = include_str!("../README.md")]
#![allow(clippy::module_name_repetitions)]

pub mod action;
pub mod local_context;
pub mod matcher;
pub mod misc;
pub mod sequence;
pub mod times;
pub mod types;

pub use murf_macros::{expect_call, expect_method_call, mock};
pub use once_cell::sync::Lazy;

pub use action::Action;
pub use local_context::LocalContext;
pub use matcher::Matcher;
pub use misc::{next_type_id, Expectation, Pointee, Pointer};
pub use sequence::{InSequence, Sequence, SequenceHandle};
pub use times::{Times, TimesRange};

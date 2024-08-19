//! Contains a generated example for the mocked types.

#![allow(unused)]

use crate::mock;

/// Simple test trait to generate a mocked version for.
pub trait Fuu {
    /// Simple method to generate a mocked version for.
    fn fuu(&self, x: usize) -> usize;
}

mock! {
    /// Type the implements the [`Fuu`] trait.
    #[derive(Default, Debug)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, x: usize) -> usize;
    }
}

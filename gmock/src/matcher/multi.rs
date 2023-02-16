use std::fmt::{Formatter, Result as FmtResult};
use std::mem::take;

use crate::Matcher;

pub fn multi<T>(value: T) -> Multi<T> {
    Multi(value)
}

pub struct Multi<T>(T);

macro_rules! impl_multi {
    (($( $arg_name:ident: $arg_type:ident ),+) => ($( $matcher_name:ident: $matcher_type:ident ),+)) => {
        #[allow(unused_parens)]
        impl<$( $arg_type ),+  $( , $matcher_type )+> Matcher<($( $arg_type ),+)> for Multi<($( $matcher_type ),+)>
        where
            $(
                $matcher_type: Matcher<$arg_type>,
            )+
        {
            fn matches(&self, ($( $arg_name ),+): &($( $arg_type ),+)) -> bool {
                let Self(($( $matcher_name ),+)) = self;

                $(
                    $matcher_name.matches($arg_name)
                )&&+
            }

            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                let mut first = true;
                let Self(($( $matcher_name ),+)) = self;

                $(
                    if !take(&mut first) {
                        write!(f, ", ")?;
                    }

                    $matcher_name.fmt(f)?;
                )+

                Ok(())
            }
        }
    };
}

impl_multi!((a0: T0) => (m0: M0));
impl_multi!((a0: T0, a1: T1) => (m0: M0, m1: M1));
impl_multi!((a0: T0, a1: T1, a2: T2) => (m0: M0, m1: M1, m2: M2));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3) => (m0: M0, m1: M1, m2: M2, m3: M3));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3, a4: T4) => (m0: M0, m1: M1, m2: M2, m3: M3, m4: M4));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3, a4: T4, a5: T5) => (m0: M0, m1: M1, m2: M2, m3: M3, m4: M4, m5: M5));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3, a4: T4, a5: T5, a6: T6) => (m0: M0, m1: M1, m2: M2, m3: M3, m4: M4, m5: M5, m6: M6));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3, a4: T4, a5: T5, a6: T6, a7: T7) => (m0: M0, m1: M1, m2: M2, m3: M3, m4: M4, m5: M5, m6: M6, m7: M7));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3, a4: T4, a5: T5, a6: T6, a7: T7, a8: T8) => (m0: M0, m1: M1, m2: M2, m3: M3, m4: M4, m5: M5, m6: M6, m7: M7, m8: M8));
impl_multi!((a0: T0, a1: T1, a2: T2, a3: T3, a4: T4, a5: T5, a6: T6, a7: T7, a8: T8, a9: T9) => (m0: M0, m1: M1, m2: M2, m3: M3, m4: M4, m5: M5, m6: M6, m7: M7, m8: M8, m9: M9));

#![no_std]

pub use aya_btf_map_macros as macros;

pub mod hash_map;

pub use hash_map::HashMap;

macro_rules! btf_map_def {
    ($name:ident, $t:ident) => {
        #[allow(dead_code)]
        pub struct $name<K, V, const M: usize, const F: usize = 0> {
            r#type: *const [i32; $t as usize],
            key: *const K,
            value: *const V,
            max_entries: *const [i32; M],
            map_flags: *const [i32; F],
        }

        impl<K, V, const M: usize, const F: usize> $name<K, V, M, F> {
            pub const fn new() -> $name<K, V, M, F> {
                $name {
                    r#type: &[0i32; $t as usize] as *const _,
                    key: ::core::ptr::null(),
                    value: ::core::ptr::null(),
                    max_entries: &[0i32; M] as *const _,
                    map_flags: &[0i32; F] as *const _,
                }
            }
        }
    };
}

pub(crate) use btf_map_def;

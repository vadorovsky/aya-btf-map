use core::ptr::NonNull;

use aya_bpf::{
    bindings::bpf_map_type::BPF_MAP_TYPE_HASH,
    cty::{c_long, c_void},
    helpers::{bpf_map_delete_elem, bpf_map_lookup_elem, bpf_map_update_elem},
};

use crate::btf_map_def;

btf_map_def!(HashMap, BPF_MAP_TYPE_HASH);

impl<K, V, const M: usize, const F: usize> HashMap<K, V, M, F> {
    #[inline]
    pub unsafe fn get(&mut self, key: &K) -> Option<&V> {
        get(self, key)
    }

    #[inline]
    pub fn get_ptr(&mut self, key: &K) -> Option<*const V> {
        get_ptr(self, key)
    }

    #[inline]
    pub fn get_ptr_mut(&mut self, key: &K) -> Option<*mut V> {
        get_ptr_mut(self, key)
    }

    #[inline]
    pub fn insert(&mut self, key: &K, value: &V, flags: u64) -> Result<(), c_long> {
        insert(self, key, value, flags)
    }

    #[inline]
    pub fn remove(&mut self, key: &K) -> Result<(), c_long> {
        remove(self, key)
    }
}

#[inline]
fn get_ptr_mut<K, V, T>(def: &mut T, key: &K) -> Option<*mut V> {
    unsafe {
        let value = bpf_map_lookup_elem(def as *mut T as *mut _, key as *const _ as *const c_void);
        // FIXME: alignment
        NonNull::new(value as *mut V).map(|p| p.as_ptr())
    }
}

#[inline]
fn get_ptr<K, V, T>(def: &mut T, key: &K) -> Option<*const V> {
    get_ptr_mut(def, key).map(|p| p as *const V)
}

#[inline]
unsafe fn get<'a, K, V, T>(def: &mut T, key: &K) -> Option<&'a V> {
    get_ptr(def, key).map(|p| &*p)
}

#[inline]
fn insert<K, V, T>(def: *mut T, key: &K, value: &V, flags: u64) -> Result<(), c_long> {
    let ret = unsafe {
        bpf_map_update_elem(
            def as *mut _,
            key as *const _ as *const _,
            value as *const _ as *const _,
            flags,
        )
    };
    (ret == 0).then_some(()).ok_or(ret)
}

#[inline]
fn remove<K, T>(def: *mut T, key: &K) -> Result<(), c_long> {
    let ret = unsafe { bpf_map_delete_elem(def as *mut _, key as *const _ as *const c_void) };
    (ret == 0).then_some(()).ok_or(ret)
}

use aya_bpf::bindings::bpf_map_type::BPF_MAP_TYPE_HASH;

use crate::btf_map_def;

btf_map_def!(HashMap, BPF_MAP_TYPE_HASH);

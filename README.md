# aya-btf-map

**Experimental** BTF map support for Aya. Requires using custom LLVM and#
bpf-linker builds.

This crate provides macros and structures for eBPF programs for using BTF maps
[[1]](https://github.com/libbpf/libbpf/issues/355)
[[2]](https://lwn.net/Articles/790177/) instead of
[legacy BPF maps](https://github.com/libbpf/libbpf/wiki/Libbpf:-the-road-to-v1.0#drop-support-for-legacy-bpf-map-declaration-syntax)
currently supported by the main [aya-bpf](https://github.com/aya-rs/aya/tree/main/bpf/aya-bpf)
crate.

Using this crate allows to emit the BTF debug info for your eBPF programs
written in Rust.

## Prerequisites

### LLVM (with custom patches)

You need to use [this fork and branch of LLVM](https://github.com/vadorovsky/llvm-project/tree/bpf-fixes).

After you clone it somewhere and enter its directory, build LLVM with the
following commands:

WARNING! This example with debug build requires at least 32 GB RAM to build in
reasonable time.

```
mkdir build
cd build

CC=clang CXX=clang++ cmake -DCMAKE_BUILD_TYPE=Debug -DLLVM_PARALLEL_LINK_JOBS=1 -DLLVM_ENABLE_LLD=1 -DLLVM_BUILD_LLVM_DYLIB=1 -GNinja ../llvm/
ninja
```

`LLVM_PARALLEL_LINK_JOBS` ensures that linking is done with only 1 core. Using
lld and clang(++) makes the build faster.

If you encounter any problems with OOM killer or your machine being unusable,
you can trim down the number of ninja threads:

```
ninja -j[number_of_threads]
```

It's also helpful to resize the Swap to match your RAM size and use above command with ``` -l 1 ``` to reduce overhead on the CPU usage because of expensive linking. That way the build is parallel with sequential linking.

If you still have problems or have less than 64GB, try a release build:

```
CC=clang CXX=clang++ cmake -DCMAKE_BUILD_TYPE=Release -DLLVM_PARALLEL_LINK_JOBS=1 -DLLVM_ENABLE_LLD=1 -GNinja ../llvm/
ninja
```
### bpf-linker (with custom patches)

You need to use [this fork and branch of bpf-linker](https://github.com/vadorovsky/bpf-linker/tree/fix-di).

After cloning and entering the directory, we need to install bpf-linker with
*system-llvm* feature and point to the patched build with `LLVM_SYS_160_PREFIX`
variable:

```
LLVM_SYS_160_PREFIX=[path_to_your_llvm_repo]/build cargo install --path . --no-default-features --features system-llvm bpf-linker
```

For example:

```
LLVM_SYS_160_PREFIX=/home/vadorovsky/repos/llvm-project/build cargo install --path . --no-default-features --features system-llvm bpf-linker
```

## Example

Example of a simple eBPF program using a BTF hash map:

```rust
#![no_std]
#![no_main]

use aya_bpf::{cty::c_long, macros::tracepoint, programs::TracePointContext};
use aya_btf_map::{macros::btf_map, HashMap};

#[btf_map]
static mut PID_MAP: HashMap<i32, i32, 1024> = HashMap::new();

#[tracepoint(name = "fork")]
pub fn fork(ctx: TracePointContext) -> u32 {
    match try_fork(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret as u32,
    }
}

fn try_fork(ctx: TracePointContext) -> Result<u32, c_long> {
    // Load the pointer to the filename. The offset value can be found running:
    // sudo cat /sys/kernel/debug/tracing/events/sched/sched_process_fork/format
    const PARENT_PID_OFFSET: usize = 24;
    const CHILD_PID_OFFSET: usize = 44;
    let parent_pid: i32 = unsafe { ctx.read_at(PARENT_PID_OFFSET)? };
    let child_pid: i32 = unsafe { ctx.read_at(CHILD_PID_OFFSET)? };

    unsafe { PID_MAP.insert(&parent_pid, &child_pid, 0)? };

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
```

aya-btf-map needs to be added as a dependency in `Cargo.toml`:

```
[dependencies]
aya-bpf = { git = "https://github.com/aya-rs/aya", branch = "main" }
[...]
aya-btf-map = { git = "https://github.com/vadorovsky/aya-btf-map", branch = "main" }
```

## Screenshots

![aya-btf-maps](https://raw.githubusercontent.com/vadorovsky/aya-btf-map/main/assets/aya-btf-map.png)

![btf-dump](https://raw.githubusercontent.com/vadorovsky/aya-btf-map/main/assets/btf-dump.png)

![objdump](https://raw.githubusercontent.com/vadorovsky/aya-btf-map/main/assets/objdump.png)

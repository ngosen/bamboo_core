name	description
rust-performance
Use when optimizing Rust code performance. Covers profiling with flamegraph and perf, benchmarking with criterion, allocation analysis with dhat, Arc, Rc, Cow, SmallVec, arena allocation, zero-copy, SIMD, cache locality,
Performance Optimization
Profiling First
cargo install flamegraph && cargo flamegraph --bin myapp  # CPU
cargo bench                                                # Benchmarks (criterion)
valgrind --tool=cachegrind ./target/release/myapp          # Cache analysis
Rule	Guideline
Use mimalloc as global allocator in apps	M-MIMALLOC-APPS
Profile hot paths early with criterion/divan	M-HOTPATH
Optimize items/CPU-cycle, avoid empty cycles	M-THROUGHPUT
yield_now().await every 10-100us in CPU loops	M-YIELD-POINTS
Allocation Avoidance
// Cow: avoid allocation when not needed
use std::borrow::Cow;
fn to_uppercase(s: &str) -> Cow<'_, str> {
    if s.chars().all(|c| c.is_uppercase()) {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_uppercase())
    }
}

// Reuse buffers across iterations
let mut buffer = Vec::new();
for item in items {
    buffer.clear();
    process(&mut buffer, item);
}

// Pre-allocate capacity
let mut v = Vec::with_capacity(10000);
Collection Choice
Need	Collection	Notes
Sequential access	Vec<T>	Best cache locality
Random access by key	HashMap<K, V>	O(1) lookup
Ordered keys	BTreeMap<K, V>	O(log n) lookup
Small sets (<20)	Vec<T> + linear search	Lower overhead
FIFO queue	VecDeque<T>	O(1) push/pop both ends
Frequent membership check	HashSet<T>	O(1) vs O(n) for Vec
Fixed-size	[T; N] array	No heap, known size
Iterator Best Practices
// BAD: bounds checking on each access
for i in 0..vec.len() { process(vec[i]); }

// GOOD: no bounds checking, idiomatic
for item in &vec { process(item); }

// BAD: unnecessary intermediate allocation
let filtered: Vec<_> = items.iter().filter(|x| x.valid).collect();
for item in filtered { process(item); }

// GOOD: chain iterators — no allocation
for item in items.iter().filter(|x| x.valid) { process(item); }

// Counting? Don't collect.
let count = items.iter().filter(|x| x.valid).count();
String Optimization
// BAD: O(n^2) allocations in loop
let mut result = String::new();
for s in strings { result = result + &s; }

// GOOD: pre-calculate capacity
let total_len: usize = strings.iter().map(|s| s.len()).sum();
let mut result = String::with_capacity(total_len);
for s in strings { result.push_str(&s); }

// BEST: use join for simple concatenation
let result = strings.join("");

// Prefer bytes over chars when ASCII
// s.bytes() is faster than s.chars()
Parallelism with Rayon
use rayon::prelude::*;

// Sequential
let sum: i32 = (0..1_000_000).map(|x| x * x).sum();

// Parallel (automatic work stealing)
let sum: i32 = (0..1_000_000).into_par_iter().map(|x| x * x).sum();

// Parallel with custom chunk size
let results: Vec<_> = data
    .par_chunks(1000)
    .map(|chunk| process_chunk(chunk))
    .collect();
Memory Layout
// BAD: 24 bytes due to padding
struct Bad { a: u8, b: u64, c: u8 } // 1+7pad + 8 + 1+7pad

// GOOD: 16 bytes — order fields largest-first
struct Good { b: u64, a: u8, c: u8 } // 8 + 1 + 1 + 6pad

// Box large enum variants
enum Message {
    Quit,
    Data(Box<[u8; 10000]>), // pointer-sized, not 10KB
}

// Compile-time size assertion
const _: () = assert!(std::mem::size_of::<MyStruct>() <= 64);
Release Build Settings
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Slower compile, faster code
panic = "abort"      # Smaller binary, no unwinding overhead
strip = true         # Strip debug symbols
Criterion Benchmarks
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_parse(c: &mut Criterion) {
    let input = "test data".repeat(1000);
    c.bench_function("parse_v1", |b| b.iter(|| parse_v1(&input)));
    c.bench_function("parse_v2", |b| b.iter(|| parse_v2(&input)));
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);

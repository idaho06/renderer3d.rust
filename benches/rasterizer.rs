use criterion::{criterion_group, criterion_main, Criterion};

// Benchmarks are added in Phase 1, once the iteration archive is in place.
// Each bench will compare the historical v1 implementation against the final
// version for: texture sampling, scanline interpolation, color multiply, and
// buffer clear.

fn placeholder(_c: &mut Criterion) {}

criterion_group!(benches, placeholder);
criterion_main!(benches);

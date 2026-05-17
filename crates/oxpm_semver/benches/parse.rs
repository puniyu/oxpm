use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use oxpm_semver::Version;

fn bench_version_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_parse");
    group.bench_function("simple", |b| {
        b.iter(|| Version::parse(black_box("1.2.3")))
    });
    group.bench_function("prerelease", |b| {
        b.iter(|| Version::parse(black_box("1.0.0-alpha.1")))
    });
    group.bench_function("build_metadata", |b| {
        b.iter(|| Version::parse(black_box("1.0.0+build.123")))
    });
    group.bench_function("full", |b| {
        b.iter(|| Version::parse(black_box("1.0.0-beta.2+build.456")))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_version_parse
);
criterion_main!(benches);
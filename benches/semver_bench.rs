//! Benchmarks for semver-php crate.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use semver_php::{Semver, VersionParser};

// Test data sets
const VERSIONS_SIMPLE: &[&str] = &[
	"1.0.0",
	"2.0.0",
	"1.2.3",
	"0.1.0",
	"10.20.30",
	"1.0.0-alpha",
	"1.0.0-beta",
	"1.0.0-RC1",
];

const VERSIONS_COMPLEX: &[&str] = &[
	"1.0.0",
	"v2.0.0",
	"1.2.3.4",
	"1.0.0-alpha1",
	"1.0.0-beta2",
	"1.0.0-RC1",
	"dev-master",
	"1.x-dev",
	"2.0.x-dev",
	"1.0.0+build.123",
	"2010.01.02",
	"1.0.0-dev",
];

const CONSTRAINTS_SIMPLE: &[&str] = &[
	"^1.0", "~1.2", ">=1.0", "<2.0", "1.0.*", "1.*", "*", "1.0.0",
];

const CONSTRAINTS_COMPLEX: &[&str] = &[
	"^1.0 || ^2.0",
	">=1.0 <2.0",
	"~1.2.3",
	"^0.2.3",
	"1.0 - 2.0",
	">=1.0, <2.0, !=1.5.0",
	"^1.0 || ^2.0 || ^3.0",
	">=1.0 <1.1 || >=1.2 <1.3",
];

const SATISFIES_CASES: &[(&str, &str)] = &[
	("1.2.3", "^1.0"),
	("2.0.0", "^1.0 || ^2.0"),
	("1.5.0", ">=1.0 <2.0"),
	("1.2.5", "~1.2.3"),
	("0.2.9", "^0.2.3"),
	("1.5.0", "1.0 - 2.0"),
	("1.2.3", "1.2.*"),
	("2.0.0", "*"),
	("1.0.0-alpha", ">=1.0.0-alpha <1.0.0"),
	("1.0.0-RC1", "^1.0@RC"),
];

const SORT_VERSIONS: &[&str] = &[
	"2.0.0",
	"1.0.0",
	"1.5.0",
	"0.1.0",
	"1.0.0-alpha",
	"1.0.0-beta",
	"1.0.0-RC1",
	"3.0.0",
	"1.2.3",
	"10.0.0",
];

fn bench_normalize(c: &mut Criterion) {
	let mut group = c.benchmark_group("normalize");

	// Simple versions
	group.throughput(Throughput::Elements(VERSIONS_SIMPLE.len() as u64));
	group.bench_function("simple", |b| {
		b.iter(|| {
			for version in VERSIONS_SIMPLE {
				black_box(VersionParser::normalize(black_box(version)).unwrap());
			}
		})
	});

	// Complex versions
	group.throughput(Throughput::Elements(VERSIONS_COMPLEX.len() as u64));
	group.bench_function("complex", |b| {
		b.iter(|| {
			for version in VERSIONS_COMPLEX {
				black_box(VersionParser::normalize(black_box(version)).unwrap());
			}
		})
	});

	// Single versions at different complexity
	group.throughput(Throughput::Elements(1));
	for version in ["1.0.0", "1.0.0-alpha1", "dev-master", "1.x-dev"] {
		group.bench_with_input(BenchmarkId::new("single", version), &version, |b, v| {
			b.iter(|| black_box(VersionParser::normalize(black_box(v)).unwrap()))
		});
	}

	group.finish();
}

fn bench_parse_constraints(c: &mut Criterion) {
	let mut group = c.benchmark_group("parse_constraints");

	// Simple constraints
	group.throughput(Throughput::Elements(CONSTRAINTS_SIMPLE.len() as u64));
	group.bench_function("simple", |b| {
		b.iter(|| {
			for constraint in CONSTRAINTS_SIMPLE {
				black_box(VersionParser::parse_constraints(black_box(constraint)).unwrap());
			}
		})
	});

	// Complex constraints
	group.throughput(Throughput::Elements(CONSTRAINTS_COMPLEX.len() as u64));
	group.bench_function("complex", |b| {
		b.iter(|| {
			for constraint in CONSTRAINTS_COMPLEX {
				black_box(VersionParser::parse_constraints(black_box(constraint)).unwrap());
			}
		})
	});

	// Single constraints at different complexity
	group.throughput(Throughput::Elements(1));
	for constraint in ["^1.0", ">=1.0 <2.0", "^1.0 || ^2.0 || ^3.0"] {
		group.bench_with_input(
			BenchmarkId::new("single", constraint),
			&constraint,
			|b, c| b.iter(|| black_box(VersionParser::parse_constraints(black_box(c)).unwrap())),
		);
	}

	group.finish();
}

fn bench_satisfies(c: &mut Criterion) {
	let mut group = c.benchmark_group("satisfies");

	// All test cases
	group.throughput(Throughput::Elements(SATISFIES_CASES.len() as u64));
	group.bench_function("all_cases", |b| {
		b.iter(|| {
			for (version, constraint) in SATISFIES_CASES {
				black_box(Semver::satisfies(black_box(version), black_box(constraint)).unwrap());
			}
		})
	});

	// Single satisfies checks
	group.throughput(Throughput::Elements(1));
	for (version, constraint) in [
		("1.2.3", "^1.0"),
		("2.0.0", "^1.0 || ^2.0"),
		("1.5.0", ">=1.0 <2.0"),
	] {
		let id = format!("{} vs {}", version, constraint);
		group.bench_with_input(
			BenchmarkId::new("single", &id),
			&(version, constraint),
			|b, (v, c)| {
				b.iter(|| black_box(Semver::satisfies(black_box(v), black_box(c)).unwrap()))
			},
		);
	}

	group.finish();
}

fn bench_sort(c: &mut Criterion) {
	let mut group = c.benchmark_group("sort");

	// Sort 10 versions
	group.throughput(Throughput::Elements(SORT_VERSIONS.len() as u64));
	group.bench_function("10_versions", |b| {
		b.iter(|| black_box(Semver::sort(black_box(SORT_VERSIONS)).unwrap()))
	});

	// Sort 100 versions (duplicated set)
	let versions_100: Vec<&str> = SORT_VERSIONS.iter().cycle().take(100).copied().collect();
	group.throughput(Throughput::Elements(100));
	group.bench_function("100_versions", |b| {
		b.iter(|| black_box(Semver::sort(black_box(&versions_100)).unwrap()))
	});

	group.finish();
}

fn bench_satisfied_by(c: &mut Criterion) {
	let mut group = c.benchmark_group("satisfied_by");

	let versions: Vec<&str> = vec![
		"0.5.0", "1.0.0", "1.0.5", "1.1.0", "1.2.0", "1.5.0", "2.0.0", "2.5.0", "3.0.0",
	];

	group.throughput(Throughput::Elements(versions.len() as u64));

	for constraint in ["^1.0", ">=1.0 <2.0", "^1.0 || ^2.0"] {
		group.bench_with_input(
			BenchmarkId::new("filter", constraint),
			&(&versions, constraint),
			|b, (v, c)| {
				b.iter(|| black_box(Semver::satisfied_by(black_box(v), black_box(c)).unwrap()))
			},
		);
	}

	group.finish();
}

fn bench_is_valid(c: &mut Criterion) {
	let mut group = c.benchmark_group("is_valid");

	let valid_versions = VERSIONS_COMPLEX;
	let invalid_versions: &[&str] = &["not-a-version", "abc", "1.2.3.4.5.6", ">>>1.0"];
	let mixed: Vec<&str> = valid_versions
		.iter()
		.chain(invalid_versions.iter())
		.copied()
		.collect();

	group.throughput(Throughput::Elements(mixed.len() as u64));
	group.bench_function("mixed", |b| {
		b.iter(|| {
			for version in &mixed {
				black_box(VersionParser::is_valid(black_box(version)));
			}
		})
	});

	group.finish();
}

fn bench_parse_stability(c: &mut Criterion) {
	let mut group = c.benchmark_group("parse_stability");

	let versions = &[
		"1.0.0",
		"1.0.0-dev",
		"1.0.0-alpha1",
		"1.0.0-beta2",
		"1.0.0-RC1",
		"dev-master",
		"1.x-dev",
	];

	group.throughput(Throughput::Elements(versions.len() as u64));
	group.bench_function("all", |b| {
		b.iter(|| {
			for version in versions {
				black_box(VersionParser::parse_stability(black_box(version)));
			}
		})
	});

	group.finish();
}

criterion_group!(
	benches,
	bench_normalize,
	bench_parse_constraints,
	bench_satisfies,
	bench_sort,
	bench_satisfied_by,
	bench_is_valid,
	bench_parse_stability,
);

criterion_main!(benches);

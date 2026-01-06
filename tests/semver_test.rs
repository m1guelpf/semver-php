//! Ported from PHP SemverTest.php
//! Tests for the Semver facade

use semver_php::Semver;

/// Positive satisfies cases: version satisfies constraint
const SATISFIES_POSITIVE_CASES: &[(&str, &str)] = &[
	("1.2.3", "1.0.0 - 2.0.0"),
	("1.2.3", "^1.2.3+build"),
	("1.3.0", "^1.2.3+build"),
	("2.4.3-alpha", "1.2.3+asdf - 2.4.3+asdf"),
	("1.3.0-beta", ">1.2"),
	("1.2.3-beta", "<=1.2.3"),
	("1.2.3-beta", "^1.2.3"),
	("1.2.3", "1.2.3+asdf - 2.4.3+asdf"),
	("1.0.0", "1.0.0"),
	("1.2.3", "*"),
	("v1.2.3", "*"),
	("1.0.0", ">=1.0.0"),
	("1.0.1", ">=1.0.0"),
	("1.1.0", ">=1.0.0"),
	("1.0.1", ">1.0.0"),
	("1.1.0", ">1.0.0"),
	("2.0.0", "<=2.0.0"),
	("1.9999.9999", "<=2.0.0"),
	("0.2.9", "<=2.0.0"),
	("1.9999.9999", "<2.0.0"),
	("0.2.9", "<2.0.0"),
	("1.0.0", ">= 1.0.0"),
	("1.0.1", ">=  1.0.0"),
	("1.1.0", ">=   1.0.0"),
	("1.0.1", "> 1.0.0"),
	("1.1.0", ">  1.0.0"),
	("2.0.0", "<=   2.0.0"),
	("1.9999.9999", "<= 2.0.0"),
	("0.2.9", "<=  2.0.0"),
	("1.9999.9999", "<    2.0.0"),
	("v0.1.97", ">=0.1.97"),
	("0.1.97", ">=0.1.97"),
	("1.2.4", "0.1.20 || 1.2.4"),
	("0.0.0", ">=0.2.3 || <0.0.1"),
	("0.2.3", ">=0.2.3 || <0.0.1"),
	("0.2.4", ">=0.2.3 || <0.0.1"),
	("2.1.3", "2.x.x"),
	("1.2.3", "1.2.x"),
	("2.1.3", "1.2.x || 2.x"),
	("1.2.3", "1.2.x || 2.x"),
	("1.2.3", "x"),
	("2.1.3", "2.*.*"),
	("1.2.3", "1.2.*"),
	("2.1.3", "1.2.* || 2.*"),
	("1.2.3", "1.2.* || 2.*"),
	("1.2.3", "*"),
	("2.9.0", "~2.4"), // >=2.4.0 <3.0.0
	("2.4.5", "~2.4"),
	("1.2.3", "~1"),   // >=1.0.0 <2.0.0
	("1.4.7", "~1.0"), // >=1.0.0 <2.0.0
	("1.0.0", ">=1"),
	("1.0.0", ">= 1"),
	("1.2.8", ">1.2"), // >1.2.0
	("1.1.1", "<1.2"), // <1.2.0
	("1.1.1", "< 1.2"),
	("1.2.3", "~1.2.1 >=1.2.3"),
	("1.2.3", "~1.2.1 =1.2.3"),
	("1.2.3", "~1.2.1 1.2.3"),
	("1.2.3", "~1.2.1 >=1.2.3 1.2.3"),
	("1.2.3", "~1.2.1 1.2.3 >=1.2.3"),
	("1.2.3", "~1.2.1 1.2.3"),
	("1.2.3", ">=1.2.1 1.2.3"),
	("1.2.3", "1.2.3 >=1.2.1"),
	("1.2.3", ">=1.2.3 >=1.2.1"),
	("1.2.3", ">=1.2.1 >=1.2.3"),
	("1.2.8", ">=1.2"),
	("1.8.1", "^1.2.3"),
	("0.1.2", "^0.1.2"),
	("0.1.2", "^0.1"),
	("1.4.2", "^1.2"),
	("1.4.2", "^1.2 ^1"),
	("0.0.1-beta", "^0.0.1-alpha"),
];

/// Negative satisfies cases: version does NOT satisfy constraint
const SATISFIES_NEGATIVE_CASES: &[(&str, &str)] = &[
	("2.2.3", "1.0.0 - 2.0.0"),
	("2.0.0", "^1.2.3+build"),
	("1.2.0", "^1.2.3+build"),
	("1.0.0beta", "1"),
	("1.0.0beta", "<1"),
	("1.0.0beta", "< 1"),
	("1.0.1", "1.0.0"),
	("0.0.0", ">=1.0.0"),
	("0.0.1", ">=1.0.0"),
	("0.1.0", ">=1.0.0"),
	("0.0.1", ">1.0.0"),
	("0.1.0", ">1.0.0"),
	("3.0.0", "<=2.0.0"),
	("2.9999.9999", "<=2.0.0"),
	("2.2.9", "<=2.0.0"),
	("2.9999.9999", "<2.0.0"),
	("2.2.9", "<2.0.0"),
	("v0.1.93", ">=0.1.97"),
	("0.1.93", ">=0.1.97"),
	("1.2.3", "0.1.20 || 1.2.4"),
	("0.0.3", ">=0.2.3 || <0.0.1"),
	("0.2.2", ">=0.2.3 || <0.0.1"),
	("1.1.3", "2.x.x"),
	("3.1.3", "2.x.x"),
	("1.3.3", "1.2.x"),
	("3.1.3", "1.2.x || 2.x"),
	("1.1.3", "1.2.x || 2.x"),
	("1.1.3", "2.*.*"),
	("3.1.3", "2.*.*"),
	("1.3.3", "1.2.*"),
	("3.1.3", "1.2.* || 2.*"),
	("1.1.3", "1.2.* || 2.*"),
	("1.1.2", "2"),
	("2.4.1", "2.3"),
	("3.0.0", "~2.4"), // >=2.4.0 <3.0.0
	("2.3.9", "~2.4"),
	("0.2.3", "~1"), // >=1.0.0 <2.0.0
	("1.0.0", "<1"),
	("1.1.1", ">=1.2"),
	("2.0.0beta", "1"),
	("0.5.4-alpha", "~v0.5.4-beta"),
	("1.2.3-beta", "<1.2.3"),
	("2.0.0-alpha", "^1.2.3"),
	("1.2.2", "^1.2.3"),
	("1.1.9", "^1.2"),
];

/// satisfied_by test data: (constraint, versions, expected)
const SATISFIED_BY_CASES: &[(&str, &[&str], &[&str])] = &[
	(
		"~1.0",
		&["1.0", "1.2", "1.9999.9999", "2.0", "2.1", "0.9999.9999"],
		&["1.0", "1.2", "1.9999.9999"],
	),
	(
		">1.0 <3.0 || >=4.0",
		&[
			"1.0",
			"1.1",
			"2.9999.9999",
			"3.0",
			"3.1",
			"3.9999.9999",
			"4.0",
			"4.1",
		],
		&["1.1", "2.9999.9999", "4.0", "4.1"],
	),
	(
		"^0.2.0",
		&["0.1.1", "0.1.9999", "0.2.0", "0.2.1", "0.3.0"],
		&["0.2.0", "0.2.1"],
	),
];

/// Sort test data: (input, sorted_asc, sorted_desc)
const SORT_CASES: &[(&[&str], &[&str], &[&str])] = &[
	(
		&["1.0", "0.1", "0.1", "3.2.1", "2.4.0-alpha", "2.4.0"],
		&["0.1", "0.1", "1.0", "2.4.0-alpha", "2.4.0", "3.2.1"],
		&["3.2.1", "2.4.0", "2.4.0-alpha", "1.0", "0.1", "0.1"],
	),
	(
		&["dev-foo", "dev-master", "1.0", "50.2"],
		&["dev-foo", "1.0", "50.2", "dev-master"],
		&["dev-master", "50.2", "1.0", "dev-foo"],
	),
];

#[test]
fn test_satisfies_positive() {
	for (i, (version, constraint)) in SATISFIES_POSITIVE_CASES.iter().enumerate() {
		let result = Semver::satisfies(version, constraint);
		assert!(
			result.is_ok(),
			"Case {}: satisfies('{}', '{}') returned error: {:?}",
			i,
			version,
			constraint,
			result.err()
		);
		assert!(
			result.unwrap(),
			"Case {}: satisfies('{}', '{}') expected true",
			i,
			version,
			constraint
		);
	}
}

#[test]
fn test_satisfies_negative() {
	for (i, (version, constraint)) in SATISFIES_NEGATIVE_CASES.iter().enumerate() {
		let result = Semver::satisfies(version, constraint);
		assert!(
			result.is_ok(),
			"Case {}: satisfies('{}', '{}') returned error: {:?}",
			i,
			version,
			constraint,
			result.err()
		);
		assert!(
			!result.unwrap(),
			"Case {}: satisfies('{}', '{}') expected false",
			i,
			version,
			constraint
		);
	}
}

#[test]
fn test_satisfied_by() {
	for (i, (constraint, versions, expected)) in SATISFIED_BY_CASES.iter().enumerate() {
		let result = Semver::satisfied_by(versions, constraint);
		assert!(
			result.is_ok(),
			"Case {}: satisfied_by({:?}, '{}') returned error: {:?}",
			i,
			versions,
			constraint,
			result.err()
		);
		let result = result.unwrap();
		let expected_vec: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
		assert_eq!(
			result, expected_vec,
			"Case {}: satisfied_by({:?}, '{}') failed",
			i, versions, constraint
		);
	}
}

#[test]
fn test_sort() {
	for (i, (versions, sorted_asc, sorted_desc)) in SORT_CASES.iter().enumerate() {
		let result = Semver::sort(versions);
		assert!(
			result.is_ok(),
			"Case {}: sort({:?}) returned error: {:?}",
			i,
			versions,
			result.err()
		);
		let result = result.unwrap();
		let expected: Vec<String> = sorted_asc.iter().map(|s| s.to_string()).collect();
		assert_eq!(
			result, expected,
			"Case {}: sort({:?}) expected {:?}, got {:?}",
			i, versions, sorted_asc, result
		);

		let result = Semver::rsort(versions);
		assert!(
			result.is_ok(),
			"Case {}: rsort({:?}) returned error: {:?}",
			i,
			versions,
			result.err()
		);
		let result = result.unwrap();
		let expected: Vec<String> = sorted_desc.iter().map(|s| s.to_string()).collect();
		assert_eq!(
			result, expected,
			"Case {}: rsort({:?}) expected {:?}, got {:?}",
			i, versions, sorted_desc, result
		);
	}
}

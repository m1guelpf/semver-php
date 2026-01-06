//! Ported from PHP ComparatorTest.php
//! Tests for version comparison functions

use semver_php::{
	compare, equal_to, greater_than, greater_than_or_equal_to, less_than, less_than_or_equal_to,
	not_equal_to,
};

/// Test data: (version1, version2, expected)
const GREATER_THAN_CASES: &[(&str, &str, bool)] = &[
	("1.25.0", "1.24.0", true),
	("1.25.0", "1.25.0", false),
	("1.25.0", "1.26.0", false),
	("1.26.0", "dev-foo", true),
	("dev-foo", "dev-master", false),
	("dev-foo", "dev-bar", false),
];

const GREATER_THAN_OR_EQUAL_TO_CASES: &[(&str, &str, bool)] = &[
	("1.25.0", "1.24.0", true),
	("1.25.0", "1.25.0", true),
	("1.25.0", "1.26.0", false),
];

const LESS_THAN_CASES: &[(&str, &str, bool)] = &[
	("1.25.0", "1.24.0", false),
	("1.25.0", "1.25.0", false),
	("1.25.0", "1.26.0", true),
	("1.0.0", "1.2-dev", true),
	("dev-foo", "1.26.0", true),
	("dev-foo", "dev-master", false),
	("dev-foo", "dev-bar", false),
];

const LESS_THAN_OR_EQUAL_TO_CASES: &[(&str, &str, bool)] = &[
	("1.25.0", "1.24.0", false),
	("1.25.0", "1.25.0", true),
	("1.25.0", "1.26.0", true),
];

const EQUAL_TO_CASES: &[(&str, &str, bool)] = &[
	("1.25.0", "1.24.0", false),
	("1.25.0", "1.25.0", true),
	("1.25.0", "1.26.0", false),
	("dev-foo", "1.26.0", false),
	("dev-foo", "dev-master", false),
	("dev-foo", "dev-bar", false),
];

const NOT_EQUAL_TO_CASES: &[(&str, &str, bool)] = &[
	("1.25.0", "1.24.0", true),
	("1.25.0", "1.25.0", false),
	("1.25.0", "1.26.0", true),
];

/// Test data: (version1, operator, version2, expected)
const COMPARE_CASES: &[(&str, &str, &str, bool)] = &[
	("1.25.0", ">", "1.24.0", true),
	("1.25.0", ">", "1.25.0", false),
	("1.25.0", ">", "1.26.0", false),
	("1.25.0", ">=", "1.24.0", true),
	("1.25.0", ">=", "1.25.0", true),
	("1.25.0", ">=", "1.26.0", false),
	("1.25.0", "<", "1.24.0", false),
	("1.25.0", "<", "1.25.0", false),
	("1.25.0", "<", "1.26.0", true),
	("1.25.0-beta2.1", "<", "1.25.0-b.3", true),
	("1.25.0-b2.1", "<", "1.25.0beta.3", true),
	("1.25.0-b-2.1", "<", "1.25.0-rc", true),
	("1.25.0", "<=", "1.24.0", false),
	("1.25.0", "<=", "1.25.0", true),
	("1.25.0", "<=", "1.26.0", true),
	("1.25.0", "==", "1.24.0", false),
	("1.25.0", "==", "1.25.0", true),
	("1.25.0", "==", "1.26.0", false),
	("1.25.0-beta2.1", "==", "1.25.0-b.2.1", true),
	("1.25.0beta2.1", "==", "1.25.0-b2.1", true),
	("1.25.0", "=", "1.24.0", false),
	("1.25.0", "=", "1.25.0", true),
	("1.25.0", "=", "1.26.0", false),
	("1.25.0", "!=", "1.24.0", true),
	("1.25.0", "!=", "1.25.0", false),
	("1.25.0", "!=", "1.26.0", true),
	("1.25.0", "<>", "1.24.0", true),
	("1.25.0", "<>", "1.25.0", false),
	("1.25.0", "<>", "1.26.0", true),
];

#[test]
fn test_greater_than() {
	for (i, (v1, v2, expected)) in GREATER_THAN_CASES.iter().enumerate() {
		let result = greater_than(v1, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: greater_than('{}', '{}') expected {}, got {}",
			i, v1, v2, expected, result
		);
	}
}

#[test]
fn test_greater_than_or_equal_to() {
	for (i, (v1, v2, expected)) in GREATER_THAN_OR_EQUAL_TO_CASES.iter().enumerate() {
		let result = greater_than_or_equal_to(v1, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: greater_than_or_equal_to('{}', '{}') expected {}, got {}",
			i, v1, v2, expected, result
		);
	}
}

#[test]
fn test_less_than() {
	for (i, (v1, v2, expected)) in LESS_THAN_CASES.iter().enumerate() {
		let result = less_than(v1, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: less_than('{}', '{}') expected {}, got {}",
			i, v1, v2, expected, result
		);
	}
}

#[test]
fn test_less_than_or_equal_to() {
	for (i, (v1, v2, expected)) in LESS_THAN_OR_EQUAL_TO_CASES.iter().enumerate() {
		let result = less_than_or_equal_to(v1, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: less_than_or_equal_to('{}', '{}') expected {}, got {}",
			i, v1, v2, expected, result
		);
	}
}

#[test]
fn test_equal_to() {
	for (i, (v1, v2, expected)) in EQUAL_TO_CASES.iter().enumerate() {
		let result = equal_to(v1, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: equal_to('{}', '{}') expected {}, got {}",
			i, v1, v2, expected, result
		);
	}
}

#[test]
fn test_not_equal_to() {
	for (i, (v1, v2, expected)) in NOT_EQUAL_TO_CASES.iter().enumerate() {
		let result = not_equal_to(v1, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: not_equal_to('{}', '{}') expected {}, got {}",
			i, v1, v2, expected, result
		);
	}
}

#[test]
fn test_compare() {
	for (i, (v1, op, v2, expected)) in COMPARE_CASES.iter().enumerate() {
		let result = compare(v1, op, v2).unwrap();
		assert_eq!(
			result, *expected,
			"Case {}: compare('{}', '{}', '{}') expected {}, got {}",
			i, v1, op, v2, expected, result
		);
	}
}

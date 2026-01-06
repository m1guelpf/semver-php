//! Ported from PHP IntervalsTest.php and SubsetsTest.php
//! Tests for interval algebra

use semver_php::{Intervals, VersionParser};

/// Subset test cases: (candidate, constraint) where candidate IS a subset of constraint
const SUBSET_CASES: &[(&str, &str)] = &[
	("*", "*"),
	("1.0.0", "*"),
	("1.0.*", "*"),
	("^1.0 || ^2.0", "*"),
	("^1.0 || ^2.0", "^1.0 || ^2.0"),
	("^1.0 || ^2.0", "^1.0 || ^2.0 || ^4.0"),
	("^1.2", "^1.0 || ^2.0"),
	("1.2.3", "^1.0 || ^2.0"),
	(">= 2.1.0", ">= 2.0.0"),
	("^2.0", "<3.0.0"),
	("^3.0", "> 2.1.3"),
	("3.0.0", "<= 3.0.0"),
	("!= 3.0.0", "*"),
	(">3", ">=3"),
	("<3", "<=3"),
	("= dev-foo", "= dev-foo"),
	("!= dev-foo", "!= dev-foo"),
	("1.5.*", "^1.4"),
	("1.3.2", "1.3.0 || 1.3.1 || 1.3.2"),
	("1.3.1", "1.3.0 || 1.3.1 || 1.3.2"),
	("^1.0 || ^3.2", "^1.0 || ^3.0"),
	(">1.6", ">1.5 || >1.7"),
	("^1.1", "> 1.0.0"),
	("^1.0, ^1.2", ">=1.2"),
	("^1.0, ^1.2", "^1.2"),
];

/// Not-subset test cases: (candidate, constraint) where candidate is NOT a subset
const NOT_SUBSET_CASES: &[(&str, &str)] = &[
	("*", "1.0.0"),
	("*", "1.0.*"),
	("*", "^1.0 || ^2.0"),
	("^1.0 || ^2.0", "^1.2"),
	("^1.0 || ^2.0", "^1.0"),
	("^1.0 || ^2.0", "1.2.3"),
	("3.0.0", "^1.0 || ^2.0"),
	("3.0.0", "< 3.0.0"),
	("3.0.0", ">= 3.0.1"),
	("!= 3.0.0", "= 3.0.0"),
	("!= 3.0.0", "!= 3.0.1"),
	(">= 1.0.0", "= 1.2.3"),
	("< 2.0.0", "= 1.2.3"),
	(">3", "^2 || ^3 || >4"),
	(">=3", ">3"),
	("<=3", "<3"),
	("!= dev-foo", "!= dev-bar"),
	("!= dev-foo", "= dev-bar"),
	("1.3.3", "1.3.0 || 1.3.1 || 1.3.2"),
	("1.3.1 || 1.3.2", "1.3.1"),
	("^1.0 || ^3.2", "^1.2 || ^3.0"),
	("^1.0 || ^3.2", "^3.0"),
];

/// Have intersections positive cases
const HAVE_INTERSECTIONS_POSITIVE: &[(&str, &str)] = &[
	("^1.0", "^1.2"),
	(">=1.0", "<2.0"),
	("*", "1.0.0"),
	("!= 1.0", "^1.0"),
	("dev-master", "dev-master"),
	("!= dev-foo", "!= dev-bar"),
];

/// Have intersections negative cases
const HAVE_INTERSECTIONS_NEGATIVE: &[(&str, &str)] = &[
	("^1.0", "^2.0"),
	(">=2.0", "<1.0"),
	("1.0.0", "2.0.0"),
	("dev-master", "dev-foo"),
	("dev-master", "^1.0"),
];

#[test]
fn test_is_subset_of() {
	for (i, (candidate, constraint)) in SUBSET_CASES.iter().enumerate() {
		let mut intervals = Intervals::new();

		let cand = VersionParser::parse_constraints(candidate).unwrap();
		let cons = VersionParser::parse_constraints(constraint).unwrap();

		assert!(
			intervals.is_subset_of(cand.as_ref(), cons.as_ref()),
			"Case {i}: '{candidate}' should be a subset of '{constraint}'",
		);
	}
}

#[test]
fn test_is_not_subset_of() {
	for (i, (candidate, constraint)) in NOT_SUBSET_CASES.iter().enumerate() {
		let mut intervals = Intervals::new();

		let cand = VersionParser::parse_constraints(candidate).unwrap();
		let cons = VersionParser::parse_constraints(constraint).unwrap();

		assert!(
			!intervals.is_subset_of(cand.as_ref(), cons.as_ref()),
			"Case {i}: '{candidate}' should NOT be a subset of '{constraint}'",
		);
	}
}

#[test]
fn test_have_intersections_positive() {
	for (i, (a, b)) in HAVE_INTERSECTIONS_POSITIVE.iter().enumerate() {
		let mut intervals = Intervals::new();

		let cons_a = VersionParser::parse_constraints(a).unwrap();
		let cons_b = VersionParser::parse_constraints(b).unwrap();

		assert!(
			intervals.have_intersections(cons_a.as_ref(), cons_b.as_ref()),
			"Case {i}: '{a}' and '{b}' should have intersections",
		);
	}
}

#[test]
fn test_have_intersections_negative() {
	for (i, (a, b)) in HAVE_INTERSECTIONS_NEGATIVE.iter().enumerate() {
		let mut intervals = Intervals::new();

		let cons_a = VersionParser::parse_constraints(a).unwrap();
		let cons_b = VersionParser::parse_constraints(b).unwrap();

		assert!(
			!intervals.have_intersections(cons_a.as_ref(), cons_b.as_ref()),
			"Case {i}: '{a}' and '{b}' should NOT have intersections",
		);
	}
}

#[test]
fn test_get_intervals_simple() {
	let mut intervals = Intervals::new();

	// Test ^1.0 produces interval [>=1.0.0.0-dev, <2.0.0.0-dev)
	let constraint = VersionParser::parse_constraints("^1.0").unwrap();
	let result = intervals.get(constraint.as_ref());

	assert_eq!(result.numeric.len(), 1);
	assert!(result.branches.matches_none());
}

#[test]
fn test_get_intervals_disjunctive() {
	let mut intervals = Intervals::new();

	// Test ^1.0 || ^3.0 produces two intervals
	let constraint = VersionParser::parse_constraints("^1.0 || ^3.0").unwrap();
	let result = intervals.get(constraint.as_ref());

	assert_eq!(
		result.numeric.len(),
		2,
		"Expected 2 intervals for '^1.0 || ^3.0'"
	);
}

#[test]
fn test_get_intervals_dev_branch() {
	let mut intervals = Intervals::new();

	// Test dev-master produces no numeric interval but has branch
	let constraint = VersionParser::parse_constraints("dev-master").unwrap();
	let result = intervals.get(constraint.as_ref());

	assert!(result.numeric.is_empty());
	assert!(!result.branches.exclude);
	assert_eq!(result.branches.names, vec!["dev-master"]);
}

#[test]
fn test_get_intervals_not_dev() {
	let mut intervals = Intervals::new();

	// Test != dev-foo produces full numeric interval and excludes the branch
	let constraint = VersionParser::parse_constraints("!= dev-foo").unwrap();
	let result = intervals.get(constraint.as_ref());

	assert_eq!(result.numeric.len(), 1);
	assert!(result.branches.exclude);
	assert_eq!(result.branches.names, vec!["dev-foo"]);
}

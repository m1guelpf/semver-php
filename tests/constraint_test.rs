//! Ported from PHP ConstraintTest.php
//! Tests for single constraint matching logic

use semver_php::{Constraint, Operator, SingleConstraint};

/// Successful version match cases from PHP's successfulVersionMatches()
/// Format: (require_op, require_ver, provide_op, provide_ver)
const SUCCESSFUL_MATCHES: &[(&str, &str, &str, &str)] = &[
	// == operator tests
	("==", "2", "==", "2"),
	("==", "2", "<", "3"),
	("==", "2", "<=", "2"),
	("==", "2", "<=", "3"),
	("==", "2", ">=", "1"),
	("==", "2", ">=", "2"),
	("==", "2", ">", "1"),
	("==", "2", "!=", "1"),
	("==", "2", "!=", "3"),
	// < operator tests
	("<", "2", "==", "1"),
	("<", "2", "<", "1"),
	("<", "2", "<", "2"),
	("<", "2", "<", "3"),
	("<", "2", "<=", "1"),
	("<", "2", "<=", "2"),
	("<", "2", "<=", "3"),
	("<", "2", ">=", "1"),
	("<", "2", ">", "1"),
	("<", "2", "!=", "1"),
	("<", "2", "!=", "2"),
	("<", "2", "!=", "3"),
	// <= operator tests
	("<=", "2", "==", "1"),
	("<=", "2", "==", "2"),
	("<=", "2", "<", "1"),
	("<=", "2", "<", "2"),
	("<=", "2", "<", "3"),
	("<=", "2", "<=", "1"),
	("<=", "2", "<=", "2"),
	("<=", "2", "<=", "3"),
	("<=", "2", ">=", "1"),
	("<=", "2", ">=", "2"),
	("<=", "2", ">", "1"),
	("<=", "2", "!=", "1"),
	("<=", "2", "!=", "2"),
	("<=", "2", "!=", "3"),
	// >= operator tests
	(">=", "2", "==", "2"),
	(">=", "2", "==", "3"),
	(">=", "2", "<", "3"),
	(">=", "2", "<=", "2"),
	(">=", "2", "<=", "3"),
	(">=", "2", ">=", "1"),
	(">=", "2", ">=", "2"),
	(">=", "2", ">=", "3"),
	(">=", "2", ">", "1"),
	(">=", "2", ">", "2"),
	(">=", "2", ">", "3"),
	(">=", "2", "!=", "1"),
	(">=", "2", "!=", "2"),
	(">=", "2", "!=", "3"),
	// > operator tests
	(">", "2", "==", "3"),
	(">", "2", "<", "3"),
	(">", "2", "<=", "3"),
	(">", "2", ">=", "1"),
	(">", "2", ">=", "2"),
	(">", "2", ">=", "3"),
	(">", "2", ">", "1"),
	(">", "2", ">", "2"),
	(">", "2", ">", "3"),
	(">", "2", "!=", "1"),
	(">", "2", "!=", "2"),
	(">", "2", "!=", "3"),
	// != operator tests
	("!=", "2", "!=", "1"),
	("!=", "2", "!=", "2"),
	("!=", "2", "!=", "3"),
	("!=", "2", "==", "1"),
	("!=", "2", "==", "3"),
	("!=", "2", "<", "1"),
	("!=", "2", "<", "2"),
	("!=", "2", "<", "3"),
	("!=", "2", "<=", "1"),
	("!=", "2", "<=", "2"),
	("!=", "2", "<=", "3"),
	("!=", "2", ">=", "1"),
	("!=", "2", ">=", "2"),
	("!=", "2", ">=", "3"),
	("!=", "2", ">", "1"),
	("!=", "2", ">", "2"),
	("!=", "2", ">", "3"),
	// Branch names
	("==", "dev-foo-bar", "==", "dev-foo-bar"),
	("==", "dev-events+issue-17", "==", "dev-events+issue-17"),
	("==", "dev-foo-bar", "!=", "dev-foo-xyz"),
	("!=", "dev-foo-bar", "!=", "dev-foo-xyz"),
	// Numbers vs branches
	("==", "0.12", "!=", "dev-foo"),
	("<", "0.12", "!=", "dev-foo"),
	("<=", "0.12", "!=", "dev-foo"),
	(">=", "0.12", "!=", "dev-foo"),
	(">", "0.12", "!=", "dev-foo"),
	("!=", "0.12", "==", "dev-foo"),
	("!=", "0.12", "!=", "dev-foo"),
];

/// Failing version match cases from PHP's failingVersionMatches()
const FAILING_MATCHES: &[(&str, &str, &str, &str)] = &[
	// == operator failures
	("==", "2", "==", "1"),
	("==", "2", "==", "3"),
	("==", "2", "<", "1"),
	("==", "2", "<", "2"),
	("==", "2", "<=", "1"),
	("==", "2", ">=", "3"),
	("==", "2", ">", "2"),
	("==", "2", ">", "3"),
	("==", "2", "!=", "2"),
	// < operator failures
	("<", "2", "==", "2"),
	("<", "2", "==", "3"),
	("<", "2", ">=", "2"),
	("<", "2", ">=", "3"),
	("<", "2", ">", "2"),
	("<", "2", ">", "3"),
	// <= operator failures
	("<=", "2", "==", "3"),
	("<=", "2", ">=", "3"),
	("<=", "2", ">", "2"),
	("<=", "2", ">", "3"),
	// >= operator failures
	(">=", "2", "==", "1"),
	(">=", "2", "<", "1"),
	(">=", "2", "<", "2"),
	(">=", "2", "<=", "1"),
	// > operator failures
	(">", "2", "==", "1"),
	(">", "2", "==", "2"),
	(">", "2", "<", "1"),
	(">", "2", "<", "2"),
	(">", "2", "<=", "1"),
	(">", "2", "<=", "2"),
	// != operator failures
	("!=", "2", "==", "2"),
	// Beta version comparison
	("==", "2.0-b2", "<", "2.0-beta2"),
	("==", "dev-foo-dist", "==", "dev-foo-zist"),
	// Different branch names
	("==", "dev-foo-bar", "==", "dev-foo-xyz"),
	("==", "dev-foo-bar", "<", "dev-foo-xyz"),
	("==", "dev-foo-bar", "<=", "dev-foo-xyz"),
	("==", "dev-foo-bar", ">=", "dev-foo-xyz"),
	("==", "dev-foo-bar", ">", "dev-foo-xyz"),
	("<", "dev-foo-bar", "==", "dev-foo-xyz"),
	("<", "dev-foo-bar", "<", "dev-foo-xyz"),
	("<", "dev-foo-bar", "<=", "dev-foo-xyz"),
	("<", "dev-foo-bar", ">=", "dev-foo-xyz"),
	("<", "dev-foo-bar", ">", "dev-foo-xyz"),
	("<", "dev-foo-bar", "!=", "dev-foo-xyz"),
	("<=", "dev-foo-bar", "==", "dev-foo-xyz"),
	("<=", "dev-foo-bar", "<", "dev-foo-xyz"),
	("<=", "dev-foo-bar", "<=", "dev-foo-xyz"),
	("<=", "dev-foo-bar", ">=", "dev-foo-xyz"),
	("<=", "dev-foo-bar", ">", "dev-foo-xyz"),
	("<=", "dev-foo-bar", "!=", "dev-foo-xyz"),
	(">=", "dev-foo-bar", "==", "dev-foo-xyz"),
	(">=", "dev-foo-bar", "<", "dev-foo-xyz"),
	(">=", "dev-foo-bar", "<=", "dev-foo-xyz"),
	(">=", "dev-foo-bar", ">=", "dev-foo-xyz"),
	(">=", "dev-foo-bar", ">", "dev-foo-xyz"),
	(">=", "dev-foo-bar", "!=", "dev-foo-xyz"),
	(">", "dev-foo-bar", "==", "dev-foo-xyz"),
	(">", "dev-foo-bar", "<", "dev-foo-xyz"),
	(">", "dev-foo-bar", "<=", "dev-foo-xyz"),
	(">", "dev-foo-bar", ">=", "dev-foo-xyz"),
	(">", "dev-foo-bar", ">", "dev-foo-xyz"),
	(">", "dev-foo-bar", "!=", "dev-foo-xyz"),
	// Same branch names with non-== operators
	("==", "dev-foo-bar", "<", "dev-foo-bar"),
	("==", "dev-foo-bar", "<=", "dev-foo-bar"),
	("==", "dev-foo-bar", ">=", "dev-foo-bar"),
	("==", "dev-foo-bar", ">", "dev-foo-bar"),
	("==", "dev-foo-bar", "!=", "dev-foo-bar"),
	("<", "dev-foo-bar", "==", "dev-foo-bar"),
	("<", "dev-foo-bar", "<", "dev-foo-bar"),
	("<", "dev-foo-bar", "<=", "dev-foo-bar"),
	("<", "dev-foo-bar", ">=", "dev-foo-bar"),
	("<", "dev-foo-bar", ">", "dev-foo-bar"),
	("<", "dev-foo-bar", "!=", "dev-foo-bar"),
	("<=", "dev-foo-bar", "==", "dev-foo-bar"),
	("<=", "dev-foo-bar", "<", "dev-foo-bar"),
	("<=", "dev-foo-bar", "<=", "dev-foo-bar"),
	("<=", "dev-foo-bar", ">=", "dev-foo-bar"),
	("<=", "dev-foo-bar", ">", "dev-foo-bar"),
	("<=", "dev-foo-bar", "!=", "dev-foo-bar"),
	(">=", "dev-foo-bar", "==", "dev-foo-bar"),
	(">=", "dev-foo-bar", "<", "dev-foo-bar"),
	(">=", "dev-foo-bar", "<=", "dev-foo-bar"),
	(">=", "dev-foo-bar", ">=", "dev-foo-bar"),
	(">=", "dev-foo-bar", ">", "dev-foo-bar"),
	(">=", "dev-foo-bar", "!=", "dev-foo-bar"),
	(">", "dev-foo-bar", "==", "dev-foo-bar"),
	(">", "dev-foo-bar", "<", "dev-foo-bar"),
	(">", "dev-foo-bar", "<=", "dev-foo-bar"),
	(">", "dev-foo-bar", ">=", "dev-foo-bar"),
	(">", "dev-foo-bar", ">", "dev-foo-bar"),
	(">", "dev-foo-bar", "!=", "dev-foo-bar"),
	// Branch vs number - not comparable
	("==", "0.12", "==", "dev-foo"),
	("==", "0.12", "<", "dev-foo"),
	("==", "0.12", "<=", "dev-foo"),
	("==", "0.12", ">=", "dev-foo"),
	("==", "0.12", ">", "dev-foo"),
	("<", "0.12", "==", "dev-foo"),
	("<", "0.12", "<", "dev-foo"),
	("<", "0.12", "<=", "dev-foo"),
	("<", "0.12", ">=", "dev-foo"),
	("<", "0.12", ">", "dev-foo"),
	("<=", "0.12", "==", "dev-foo"),
	("<=", "0.12", "<", "dev-foo"),
	("<=", "0.12", "<=", "dev-foo"),
	("<=", "0.12", ">=", "dev-foo"),
	("<=", "0.12", ">", "dev-foo"),
	(">=", "0.12", "==", "dev-foo"),
	(">=", "0.12", "<", "dev-foo"),
	(">=", "0.12", "<=", "dev-foo"),
	(">=", "0.12", ">=", "dev-foo"),
	(">=", "0.12", ">", "dev-foo"),
	(">", "0.12", "==", "dev-foo"),
	(">", "0.12", "<", "dev-foo"),
	(">", "0.12", "<=", "dev-foo"),
	(">", "0.12", ">=", "dev-foo"),
	(">", "0.12", ">", "dev-foo"),
	("!=", "0.12", "<", "dev-foo"),
	("!=", "0.12", "<=", "dev-foo"),
	("!=", "0.12", ">=", "dev-foo"),
	("!=", "0.12", ">", "dev-foo"),
];

fn parse_op(s: &str) -> Operator {
	Operator::parse(s).unwrap()
}

#[test]
fn test_successful_version_matches() {
	for (i, (req_op, req_ver, prov_op, prov_ver)) in SUCCESSFUL_MATCHES.iter().enumerate() {
		let require = SingleConstraint::new(parse_op(req_op), *req_ver);
		let provide = SingleConstraint::new(parse_op(prov_op), *prov_ver);

		assert!(
			require.match_specific(&provide, false),
			"Case {}: Expected {} {} to match {} {}, but it didn't",
			i,
			req_op,
			req_ver,
			prov_op,
			prov_ver
		);

		// Test commutativity
		assert!(
			provide.match_specific(&require, false),
			"Case {} (commutative): Expected {} {} to match {} {}, but it didn't",
			i,
			prov_op,
			prov_ver,
			req_op,
			req_ver
		);
	}
}

#[test]
fn test_failing_version_matches() {
	for (i, (req_op, req_ver, prov_op, prov_ver)) in FAILING_MATCHES.iter().enumerate() {
		let require = SingleConstraint::new(parse_op(req_op), *req_ver);
		let provide = SingleConstraint::new(parse_op(prov_op), *prov_ver);

		assert!(
			!require.match_specific(&provide, false),
			"Case {}: Expected {} {} to NOT match {} {}, but it did",
			i,
			req_op,
			req_ver,
			prov_op,
			prov_ver
		);

		// Test commutativity
		assert!(
			!provide.match_specific(&require, false),
			"Case {} (commutative): Expected {} {} to NOT match {} {}, but it did",
			i,
			prov_op,
			prov_ver,
			req_op,
			req_ver
		);
	}
}

#[test]
fn test_constraint_display() {
	let c = SingleConstraint::new(Operator::Eq, "1.0.0.0");
	assert_eq!(c.to_string(), "== 1.0.0.0");

	let c = SingleConstraint::new(Operator::Ge, "2.0.0.0");
	assert_eq!(c.to_string(), ">= 2.0.0.0");

	let c = SingleConstraint::new(Operator::Ne, "3.0.0.0");
	assert_eq!(c.to_string(), "!= 3.0.0.0");
}

#[test]
fn test_dev_branch_detection() {
	let c = SingleConstraint::new(Operator::Eq, "dev-master");
	assert!(c.is_dev_branch());

	let c = SingleConstraint::new(Operator::Eq, "dev-feature-foo");
	assert!(c.is_dev_branch());

	let c = SingleConstraint::new(Operator::Eq, "1.0.0");
	assert!(!c.is_dev_branch());

	let c = SingleConstraint::new(Operator::Eq, "1.0.0-dev");
	assert!(!c.is_dev_branch()); // This is a version with -dev suffix, not a branch
}

#[test]
fn test_pretty_string() {
	let mut c = SingleConstraint::new(Operator::Eq, "1.0.0");
	assert_eq!(c.pretty_string(), "== 1.0.0");

	c.set_pretty_string("pretty-string".to_string());
	assert_eq!(c.pretty_string(), "pretty-string");
}

#[test]
fn test_bounds() {
	// Equal constraint
	let c = SingleConstraint::new(Operator::Eq, "1.0.0.0");
	assert_eq!(c.lower_bound().version(), "1.0.0.0");
	assert!(c.lower_bound().is_inclusive());
	assert_eq!(c.upper_bound().version(), "1.0.0.0");
	assert!(c.upper_bound().is_inclusive());

	// Greater than
	let c = SingleConstraint::new(Operator::Gt, "1.0.0.0");
	assert_eq!(c.lower_bound().version(), "1.0.0.0");
	assert!(!c.lower_bound().is_inclusive());
	assert!(c.upper_bound().is_positive_infinity());

	// Less than
	let c = SingleConstraint::new(Operator::Lt, "2.0.0.0");
	assert!(c.lower_bound().is_zero());
	assert_eq!(c.upper_bound().version(), "2.0.0.0");
	assert!(!c.upper_bound().is_inclusive());

	// Greater than or equal
	let c = SingleConstraint::new(Operator::Ge, "1.0.0.0");
	assert_eq!(c.lower_bound().version(), "1.0.0.0");
	assert!(c.lower_bound().is_inclusive());
	assert!(c.upper_bound().is_positive_infinity());

	// Less than or equal
	let c = SingleConstraint::new(Operator::Le, "2.0.0.0");
	assert!(c.lower_bound().is_zero());
	assert_eq!(c.upper_bound().version(), "2.0.0.0");
	assert!(c.upper_bound().is_inclusive());

	// Not equal
	let c = SingleConstraint::new(Operator::Ne, "1.0.0.0");
	assert!(c.lower_bound().is_zero());
	assert!(c.upper_bound().is_positive_infinity());
}

/// Matrix test - tests all combinations of versions and operators
#[test]
fn test_matrix_commutativity() {
	let versions = &["1.0", "2.0", "dev-master", "dev-foo", "3.0-b2", "3.0-beta2"];
	let operators = &["==", "!=", ">", "<", ">=", "<="];

	for req_ver in versions {
		for req_op in operators {
			for prov_ver in versions {
				for prov_op in operators {
					let require = SingleConstraint::new(parse_op(req_op), *req_ver);
					let provide = SingleConstraint::new(parse_op(prov_op), *prov_ver);

					let result1 = require.match_specific(&provide, false);
					let result2 = provide.match_specific(&require, false);

					assert_eq!(
						result1, result2,
						"Commutativity failed: {} {} vs {} {} - forward={}, reverse={}",
						req_op, req_ver, prov_op, prov_ver, result1, result2
					);
				}
			}
		}
	}
}

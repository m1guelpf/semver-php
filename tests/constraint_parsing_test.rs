//! Ported from PHP VersionParserTest.php
//! Tests for constraint parsing

use semver_php::VersionParser;

/// Simple constraint cases from PHP's simpleConstraints()
const SIMPLE_CONSTRAINT_CASES: &[(&str, &str)] = &[
	// Match any
	("*", "*"),
	("v*", ">= 0.0.0.0-dev"),
	("*.*", ">= 0.0.0.0-dev"),
	("v*.*", ">= 0.0.0.0-dev"),
	("*.x.*", ">= 0.0.0.0-dev"),
	("x.X.x.*", ">= 0.0.0.0-dev"),
	// Not equal
	("<>1.0.0", "!= 1.0.0.0"),
	("!=1.0.0", "!= 1.0.0.0"),
	// Greater than
	(">1.0.0", "> 1.0.0.0"),
	// Less than (adds -dev)
	("<1.2.3.4", "< 1.2.3.4-dev"),
	// Less/eq than
	("<=1.2.3", "<= 1.2.3.0"),
	// Great/eq than (adds -dev)
	(">=1.2.3", ">= 1.2.3.0-dev"),
	// Equals
	("=1.2.3", "== 1.2.3.0"),
	("==1.2.3", "== 1.2.3.0"),
	// No op means eq
	("1.2.3", "== 1.2.3.0"),
	// Completes version
	("=1.0", "== 1.0.0.0"),
	// Shorthand stability
	("1.2.3b5", "== 1.2.3.0-beta5"),
	("1.2.3a1", "== 1.2.3.0-alpha1"),
	("1.2.3p1234", "== 1.2.3.0-patch1234"),
	("1.2.3pl1234", "== 1.2.3.0-patch1234"),
	// Accepts spaces
	(">= 1.2.3", ">= 1.2.3.0-dev"),
	("< 1.2.3", "< 1.2.3.0-dev"),
	("> 1.2.3", "> 1.2.3.0"),
	// Accepts dev branches
	(">=dev-master", ">= dev-master"),
	("dev-master", "== dev-master"),
	("dev-feature-a", "== dev-feature-a"),
	("dev-some-fix", "== dev-some-fix"),
	("dev-CAPS", "== dev-CAPS"),
	// Ignores aliases
	("dev-master as 1.0.0", "== dev-master"),
	// Stability override
	("<1.2.3.4-stable", "< 1.2.3.4"),
	(">=1.2.3.4-stable", ">= 1.2.3.4"),
];

/// Wildcard constraint cases from PHP's wildcardConstraints()
/// Format: (input, min_expected, max_expected)
/// If min is empty, only max constraint applies
const WILDCARD_CONSTRAINT_CASES: &[(&str, &str, &str)] = &[
	("v2.*", ">= 2.0.0.0-dev", "< 3.0.0.0-dev"),
	("2.*.*", ">= 2.0.0.0-dev", "< 3.0.0.0-dev"),
	("20.*", ">= 20.0.0.0-dev", "< 21.0.0.0-dev"),
	("20.*.*", ">= 20.0.0.0-dev", "< 21.0.0.0-dev"),
	("2.0.*", ">= 2.0.0.0-dev", "< 2.1.0.0-dev"),
	("2.x", ">= 2.0.0.0-dev", "< 3.0.0.0-dev"),
	("2.x.x", ">= 2.0.0.0-dev", "< 3.0.0.0-dev"),
	("2.2.x", ">= 2.2.0.0-dev", "< 2.3.0.0-dev"),
	("2.10.X", ">= 2.10.0.0-dev", "< 2.11.0.0-dev"),
	("2.1.3.*", ">= 2.1.3.0-dev", "< 2.1.4.0-dev"),
	// 0.* cases - only upper bound
	("0.*", "", "< 1.0.0.0-dev"),
	("0.*.*", "", "< 1.0.0.0-dev"),
	("0.x", "", "< 1.0.0.0-dev"),
	("0.x.x", "", "< 1.0.0.0-dev"),
];

/// Tilde constraint cases from PHP's tildeConstraints()
const TILDE_CONSTRAINT_CASES: &[(&str, &str, &str)] = &[
	("~v1", ">= 1.0.0.0-dev", "< 2.0.0.0-dev"),
	("~1.0", ">= 1.0.0.0-dev", "< 2.0.0.0-dev"),
	("~1.0.0", ">= 1.0.0.0-dev", "< 1.1.0.0-dev"),
	("~1.2", ">= 1.2.0.0-dev", "< 2.0.0.0-dev"),
	("~1.2.3", ">= 1.2.3.0-dev", "< 1.3.0.0-dev"),
	("~1.2.3.4", ">= 1.2.3.4-dev", "< 1.2.4.0-dev"),
	("~1.2-beta", ">= 1.2.0.0-beta", "< 2.0.0.0-dev"),
	("~1.2-b2", ">= 1.2.0.0-beta2", "< 2.0.0.0-dev"),
	("~1.2-BETA2", ">= 1.2.0.0-beta2", "< 2.0.0.0-dev"),
	("~1.2.2-dev", ">= 1.2.2.0-dev", "< 1.3.0.0-dev"),
	("~1.2.2-stable", ">= 1.2.2.0", "< 1.3.0.0-dev"),
	("~201903.0", ">= 201903.0-dev", "< 201904.0.0.0-dev"),
	("~201903.0-beta", ">= 201903.0-beta", "< 201904.0.0.0-dev"),
	("~201903.0-stable", ">= 201903.0", "< 201904.0.0.0-dev"),
	(
		"~201903.205830.1-stable",
		">= 201903.205830.1",
		"< 201903.205831.0.0-dev",
	),
	(
		"~2.x-dev",
		">= 2.9999999.9999999.9999999-dev",
		"< 3.0.0.0-dev",
	),
	("~2.0.x-dev", ">= 2.0.9999999.9999999-dev", "< 2.1.0.0-dev"),
	("~2.0.3.x-dev", ">= 2.0.3.9999999-dev", "< 2.0.4.0-dev"),
	(
		"~0.x-dev",
		">= 0.9999999.9999999.9999999-dev",
		"< 1.0.0.0-dev",
	),
];

/// Caret constraint cases from PHP's caretConstraints()
const CARET_CONSTRAINT_CASES: &[(&str, &str, &str)] = &[
	("^v1", ">= 1.0.0.0-dev", "< 2.0.0.0-dev"),
	("^0", ">= 0.0.0.0-dev", "< 1.0.0.0-dev"),
	("^0.0", ">= 0.0.0.0-dev", "< 0.1.0.0-dev"),
	("^1.2", ">= 1.2.0.0-dev", "< 2.0.0.0-dev"),
	("^1.2.3-beta.2", ">= 1.2.3.0-beta2", "< 2.0.0.0-dev"),
	("^1.2.3.4", ">= 1.2.3.4-dev", "< 2.0.0.0-dev"),
	("^1.2.3", ">= 1.2.3.0-dev", "< 2.0.0.0-dev"),
	("^0.2.3", ">= 0.2.3.0-dev", "< 0.3.0.0-dev"),
	("^0.2", ">= 0.2.0.0-dev", "< 0.3.0.0-dev"),
	("^0.2.0", ">= 0.2.0.0-dev", "< 0.3.0.0-dev"),
	("^0.0.3", ">= 0.0.3.0-dev", "< 0.0.4.0-dev"),
	("^0.0.3-alpha", ">= 0.0.3.0-alpha", "< 0.0.4.0-dev"),
	("^0.0.3-dev", ">= 0.0.3.0-dev", "< 0.0.4.0-dev"),
	("^0.0.3-stable", ">= 0.0.3.0", "< 0.0.4.0-dev"),
	("^201903.0", ">= 201903.0-dev", "< 201904.0.0.0-dev"),
	("^201903.0-beta", ">= 201903.0-beta", "< 201904.0.0.0-dev"),
	(
		"^201903.205830.1-stable",
		">= 201903.205830.1",
		"< 201904.0.0.0-dev",
	),
	(
		"^2.x-dev",
		">= 2.9999999.9999999.9999999-dev",
		"< 3.0.0.0-dev",
	),
	("^2.0.*-dev", ">= 2.0.9999999.9999999-dev", "< 3.0.0.0-dev"),
	("^2.0.x-dev", ">= 2.0.9999999.9999999-dev", "< 3.0.0.0-dev"),
	("^2.0.3.x-dev", ">= 2.0.3.9999999-dev", "< 3.0.0.0-dev"),
	(
		"^0.x-dev",
		">= 0.9999999.9999999.9999999-dev",
		"< 1.0.0.0-dev",
	),
];

/// Hyphen constraint cases from PHP's hyphenConstraints()
const HYPHEN_CONSTRAINT_CASES: &[(&str, &str, &str)] = &[
	("v1 - v2", ">= 1.0.0.0-dev", "< 3.0.0.0-dev"),
	("1.2.3 - 2.3.4.5", ">= 1.2.3.0-dev", "<= 2.3.4.5"),
	("1.2-beta - 2.3", ">= 1.2.0.0-beta", "< 2.4.0.0-dev"),
	("1.2-beta - 2.3-dev", ">= 1.2.0.0-beta", "<= 2.3.0.0-dev"),
	("1.2-RC - 2.3.1", ">= 1.2.0.0-RC", "<= 2.3.1.0"),
	("1.2.3-alpha - 2.3-RC", ">= 1.2.3.0-alpha", "<= 2.3.0.0-RC"),
	("1 - 2.0", ">= 1.0.0.0-dev", "< 2.1.0.0-dev"),
	("1 - 2.1", ">= 1.0.0.0-dev", "< 2.2.0.0-dev"),
	("1.2 - 2.1.0", ">= 1.2.0.0-dev", "<= 2.1.0.0"),
	("1.3 - 2.1.3", ">= 1.3.0.0-dev", "<= 2.1.3.0"),
	(
		"2.0.3.x-dev - 3.0.3.x-dev",
		">= 2.0.3.9999999-dev",
		"<= 3.0.3.9999999-dev",
	),
	(
		"2.0.x-dev - 3.0.x-dev",
		">= 2.0.9999999.9999999-dev",
		"<= 3.0.9999999.9999999-dev",
	),
	(
		"2.x-dev - 3.x-dev",
		">= 2.9999999.9999999.9999999-dev",
		"<= 3.9999999.9999999.9999999-dev",
	),
	(
		"0.x-dev - 1.x-dev",
		">= 0.9999999.9999999.9999999-dev",
		"<= 1.9999999.9999999.9999999-dev",
	),
];

/// Complex constraint test cases
const CONSTRAINT_PROVIDER_CASES: &[(&str, &str)] = &[
	// numeric branch
	("3.x-dev", "== 3.9999999.9999999.9999999-dev"),
	("3-dev", "== 3.0.0.0-dev"),
	// non-numeric branches
	("dev-3.x", "== dev-3.x"),
	("xsd2php-dev", "== dev-xsd2php"),
	("3.next-dev", "== dev-3.next"),
	("foobar-dev", "== dev-foobar"),
	("dev-xsd2php", "== dev-xsd2php"),
	("dev-3.next", "== dev-3.next"),
	("dev-foobar", "== dev-foobar"),
	("dev-1.0.0-dev<1.0.5-dev", "== dev-1.0.0-dev<1.0.5-dev"),
	("dev-1.0.0-dev<1.0.5", "== dev-1.0.0-dev<1.0.5"),
	("foobar-dev as 2.1.0", "== dev-foobar"),
	(
		"foobar-dev as 2.1.0 || 3.5",
		"[== dev-foobar || == 3.5.0.0]",
	),
	(
		"foobar-dev as 2.1.0 || 3.5 as 1.5",
		"[== dev-foobar || == 3.5.0.0]",
	),
	("2.1.0 - 2.3-dev", "[>= 2.1.0.0-dev <= 2.3.0.0-dev]"),
	(
		"1.0 - 2.0.x-dev",
		"[>= 1.0.0.0-dev <= 2.0.9999999.9999999-dev]",
	),
	// borked typo constraints but common historically
	("^1.", "[>= 1.0.0.0-dev < 2.0.0.0-dev]"),
	("~1.", "[>= 1.0.0.0-dev < 2.0.0.0-dev]"),
	("1.2.", "== 1.2.0.0"),
	("1.2..dev", "== 1.2.0.0-dev"),
	("1.2-.dev", "== 1.2.0.0-dev"),
	("1.2_-dev", "== 1.2.0.0-dev"),
	// complex
	(
		"~2.5.9|~2.6,>=2.6.2",
		"[[>= 2.5.9.0-dev < 2.6.0.0-dev] || [>= 2.6.0.0-dev < 3.0.0.0-dev >= 2.6.2.0-dev]]",
	),
];

/// Multi-constraint variations (all should produce same result)
const MULTI_CONSTRAINT_CASES: &[&str] = &[
	">2.0,<=3.0",
	">2.0 <=3.0",
	">2.0  <=3.0",
	">2.0, <=3.0",
	">2.0 ,<=3.0",
	">2.0 , <=3.0",
	">2.0   , <=3.0",
	"> 2.0   <=  3.0",
	"> 2.0  ,  <=  3.0",
	"  > 2.0  ,  <=  3.0 ",
];

/// Failing constraint cases from PHP's failingConstraints()
const FAILING_CONSTRAINT_CASES: &[&str] = &[
	// empty
	"",
	// invalid version
	"1.0.0-meh",
	// operator abuse
	">2.0,,<=3.0",
	">2.0 ,, <=3.0",
	">2.0 ||| <=3.0",
	// leading operator
	",^1@dev || ^4@dev",
	",^1@dev",
	"|| ^1@dev",
	// trailing operator
	"^1@dev ||",
	"^1@dev ,",
	// caret+wildcard without -dev
	"^2.0.*",
	"^2.0.x",
	"^2.0.x-beta",
	"^2.*",
	"^2.x",
	"^2.x-beta",
	"^2.1.2.*",
	"^2.1.2.x",
	"^2.1.2.x-beta",
	// tilde+wildcard without -dev
	"~2.0.*",
	"~2.0.x",
	"~2.0.x-beta",
	"~2.*",
	"~2.x",
	"~2.x-beta",
	"~2.1.2.*",
	"~2.1.2.x",
	"~2.1.2.x-beta",
	// dash range with wildcard
	"1.x - 2.*",
	"2.x.x.x-dev - 3.x.x.x-dev",
	// broken constraints
	"^1.*-beta-dev",
	"^1. *-dev",
	"~1.*-beta-dev",
	"1.0.0-dev<1.0.5-dev",
	"*-dev",
	// just an operator
	"^",
	"^8 || ^",
	"~",
	"~1 ~",
];

#[test]
fn test_parse_simple_constraints() {
	for (i, (input, expected)) in SIMPLE_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		assert_eq!(
			result.unwrap().to_string(),
			*expected,
			"Case {i}: parse_constraints('{input}') failed",
		);
	}
}

#[test]
fn test_parse_wildcard_constraints() {
	for (i, (input, min, max)) in WILDCARD_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		let constraint = result.unwrap();
		let result_str = constraint.to_string();

		if min.is_empty() {
			// Only max constraint
			assert_eq!(
				result_str, *max,
				"Case {i}: parse_constraints('{input}') expected '{max}', got '{result_str}'"
			);
		} else {
			// Both min and max
			let expected = format!("[{min} {max}]");
			assert_eq!(
				result_str, expected,
				"Case {i}: parse_constraints('{input}') failed",
			);
		}
	}
}

#[test]
fn test_parse_tilde_constraints() {
	for (i, (input, min, max)) in TILDE_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		let expected = format!("[{min} {max}]");
		assert_eq!(
			result.unwrap().to_string(),
			expected,
			"Case {i}: parse_constraints('{input}') failed",
		);
	}
}

#[test]
fn test_parse_caret_constraints() {
	for (i, (input, min, max)) in CARET_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		let expected = format!("[{} {}]", min, max);
		assert_eq!(
			result.unwrap().to_string(),
			expected,
			"Case {i}: parse_constraints('{input}') failed",
		);
	}
}

#[test]
fn test_parse_hyphen_constraints() {
	for (i, (input, min, max)) in HYPHEN_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		let expected = format!("[{min} {max}]");
		assert_eq!(
			result.unwrap().to_string(),
			expected,
			"Case {i}: parse_constraints('{input}') failed",
		);
	}
}

#[test]
fn test_parse_constraint_provider_cases() {
	for (i, (input, expected)) in CONSTRAINT_PROVIDER_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		assert_eq!(
			result.unwrap().to_string(),
			*expected,
			"Case {i}: parse_constraints('{input}') failed",
		);
	}
}

#[test]
fn test_parse_multi_constraints() {
	let expected = "[> 2.0.0.0 <= 3.0.0.0]";

	for (i, input) in MULTI_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_ok(),
			"Case {i}: Failed to parse '{input}': {:?}",
			result.err()
		);
		assert_eq!(
			result.unwrap().to_string(),
			expected,
			"Case {i}: parse_constraints('{input}') failed",
		);
	}
}

#[test]
fn test_parse_failing_constraints() {
	for (i, input) in FAILING_CONSTRAINT_CASES.iter().enumerate() {
		let result = VersionParser::parse_constraints(input);
		assert!(
			result.is_err(),
			"Case {i}: Expected '{input}' to fail parsing, but got: {:?}",
			result.ok().map(|c| c.to_string())
		);
	}
}

#[test]
fn test_tilde_greater_than_error() {
	let result = VersionParser::parse_constraints("~>1.2");
	assert!(result.is_err());
	let err = result.unwrap_err();
	assert!(
		err.to_string().contains("~>"),
		"Error should mention invalid operator ~>"
	);
}

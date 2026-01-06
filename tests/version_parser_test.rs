//! Ported from PHP VersionParserTest.php
//! Tests for version parsing and normalization

use semver_php::{Stability, VersionParser};

/// Successful normalization cases from PHP's successfulNormalizedVersions()
const NORMALIZE_SUCCESS_CASES: &[(&str, &str)] = &[
	// Basic versions
	("1.0.0", "1.0.0.0"),
	("1.2.3.4", "1.2.3.4"),
	// Parse state
	("1.0.0RC1dev", "1.0.0.0-RC1-dev"),
	("1.0.0-rC15-dev", "1.0.0.0-RC15-dev"),
	("1.0.0.RC.15-dev", "1.0.0.0-RC15-dev"),
	("1.0.0-rc1", "1.0.0.0-RC1"),
	("1.0.0.pl3-dev", "1.0.0.0-patch3-dev"),
	// Forces w.x.y.z
	("1.0-dev", "1.0.0.0-dev"),
	("0", "0.0.0.0"),
	("99999", "99999.0.0.0"),
	// Parse long
	("10.4.13-beta", "10.4.13.0-beta"),
	("10.4.13beta2", "10.4.13.0-beta2"),
	("10.4.13beta.2", "10.4.13.0-beta2"),
	("v1.13.11-beta.0", "1.13.11.0-beta0"),
	("1.13.11.0-beta0", "1.13.11.0-beta0"),
	// Expand shorthand
	("10.4.13-b", "10.4.13.0-beta"),
	("10.4.13-b5", "10.4.13.0-beta5"),
	// Strip leading v
	("v1.0.0", "1.0.0.0"),
	// Date versions
	("2010.01", "2010.01.0.0"),
	("2010.01.02", "2010.01.02.0"),
	("2010.1.555", "2010.1.555.0"),
	("2010.10.200", "2010.10.200.0"),
	// CalVer
	("20230131.0.0", "20230131.0.0"),
	// Strip v/datetime
	("v20100102", "20100102"),
	// Date no delimiter
	("20100102", "20100102"),
	("20100102.0", "20100102.0"),
	("20100102.1.0", "20100102.1.0"),
	("20100102.0.3", "20100102.0.3"),
	("100000", "100000"),
	// Date with separators
	("2010-01-02", "2010.01.02"),
	("2012.06.07", "2012.06.07.0"),
	("2010-01-02.5", "2010.01.02.5"),
	("20100102-203040", "20100102.203040"),
	// Date dev
	("20100102.x-dev", "20100102.9999999.9999999.9999999-dev"),
	// Branches
	("dev-master", "dev-master"),
	("master", "dev-master"),
	("dev-trunk", "dev-trunk"),
	("1.x-dev", "1.9999999.9999999.9999999-dev"),
	("dev-feature-foo", "dev-feature-foo"),
	("DEV-FOOBAR", "dev-FOOBAR"),
	("dev-feature/foo", "dev-feature/foo"),
	("dev-feature+issue-1", "dev-feature+issue-1"),
	// Aliases
	("dev-master as 1.0.0", "dev-master"),
	(
		"dev-load-varnish-only-when-used as ^2.0",
		"dev-load-varnish-only-when-used",
	),
	(
		"dev-load-varnish-only-when-used@dev as ^2.0@dev",
		"dev-load-varnish-only-when-used",
	),
	// Stability flags
	("1.0.0+foo@dev", "1.0.0.0"),
	(
		"dev-load-varnish-only-when-used@stable",
		"dev-load-varnish-only-when-used",
	),
	// Semver metadata
	("1.0.0-beta.5+foo", "1.0.0.0-beta5"),
	("1.0.0+foo", "1.0.0.0"),
	("1.0.0-alpha.3.1+foo", "1.0.0.0-alpha3.1"),
	("1.0.0-alpha2.1+foo", "1.0.0.0-alpha2.1"),
	("1.0.0-alpha-2.1-3+foo", "1.0.0.0-alpha2.1-3"),
	("1.0.0+foo as 2.0", "1.0.0.0"),
	// Zero padding
	("00.01.03.04", "00.01.03.04"),
	("000.001.003.004", "000.001.003.004"),
	("0.000.103.204", "0.000.103.204"),
	("0700", "0700.0.0.0"),
	("041.x-dev", "041.9999999.9999999.9999999-dev"),
	("dev-041.003", "dev-041.003"),
	// Special cases
	("dev-1.0.0-dev<1.0.5-dev", "dev-1.0.0-dev<1.0.5-dev"),
	("dev-foo bar", "dev-foo bar"),
	// Space padding
	(" 1.0.0", "1.0.0.0"),
	("1.0.0 ", "1.0.0.0"),
];

/// Failing normalization cases from PHP's failingNormalizedVersions()
const NORMALIZE_FAIL_CASES: &[&str] = &[
	// Empty
	"",
	// Invalid chars
	"a",
	// Invalid type
	"1.0.0-meh",
	// Too many bits
	"1.0.0.0.0",
	// Non-dev arbitrary
	"feature-foo",
	// Metadata with space
	"1.0.0+foo bar",
	// Maven style release
	"1.0.1-SNAPSHOT",
	// Dev with less than
	"1.0.0<1.0.5-dev",
	"1.0.0-dev<1.0.5-dev",
	// Dev suffix with spaces
	"foo bar-dev",
	// Any with spaces
	"1.0 .2",
	// No version, no alias
	" as ",
	// No version, only alias
	" as 1.2",
	// Just an operator
	"^",
	"^8 || ^",
	"~",
	"~1 ~",
	// Constraint
	"~1",
	"^1",
	"1.*",
];

/// Branch normalization cases from PHP's successfulNormalizedBranches()
const NORMALIZE_BRANCH_CASES: &[(&str, &str)] = &[
	("v1.x", "1.9999999.9999999.9999999-dev"),
	("v1.*", "1.9999999.9999999.9999999-dev"),
	("v1.0", "1.0.9999999.9999999-dev"),
	("2.0", "2.0.9999999.9999999-dev"),
	("v1.0.x", "1.0.9999999.9999999-dev"),
	("v1.0.3.*", "1.0.3.9999999-dev"),
	("v2.4.0", "2.4.0.9999999-dev"),
	("2.4.4", "2.4.4.9999999-dev"),
	("master", "dev-master"),
	("trunk", "dev-trunk"),
	("feature-a", "dev-feature-a"),
	("FOOBAR", "dev-FOOBAR"),
	("feature+issue-1", "dev-feature+issue-1"),
];

/// Numeric alias prefix cases
const NUMERIC_ALIAS_PREFIX_CASES: &[(&str, Option<&str>)] = &[
	("0.x-dev", Some("0.")),
	("1.0.x-dev", Some("1.0.")),
	("1.x-dev", Some("1.")),
	("1.2.x-dev", Some("1.2.")),
	("1.2-dev", Some("1.2.")),
	("1-dev", Some("1.")),
	("dev-develop", None),
	("dev-master", None),
];

/// Valid version cases
const IS_VALID_CASES: &[(&str, bool)] = &[
	("0.x-dev", true),
	("dev-develop", true),
	("1.0.2", true),
	("1.0.2.5", true),
	("1.0.2.5.5", false),
	("foo", false),
];

/// Stability parsing cases
const STABILITY_CASES: &[(&str, &str)] = &[
	("1", "stable"),
	("1.0", "stable"),
	("3.2.1", "stable"),
	("v3.2.1", "stable"),
	("v2.0.x-dev", "dev"),
	("v2.0.x-dev#abc123", "dev"),
	("v2.0.x-dev#trunk/@123", "dev"),
	("3.0-RC2", "RC"),
	("dev-master", "dev"),
	("3.1.2-dev", "dev"),
	("dev-feature+issue-1", "dev"),
	("3.1.2-p1", "stable"),
	("3.1.2-pl2", "stable"),
	("3.1.2-patch", "stable"),
	("3.1.2-alpha5", "alpha"),
	("3.1.2-beta", "beta"),
	("2.0B1", "beta"),
	("1.2.0a1", "alpha"),
	("1.2_a1", "alpha"),
	("2.0.0rc1", "RC"),
	("1.0.0-alpha11+cs-1.1.0", "alpha"),
];

#[test]
fn test_normalize_succeeds() {
	for (i, (input, expected)) in NORMALIZE_SUCCESS_CASES.iter().enumerate() {
		let result = VersionParser::normalize(input);
		assert!(
			result.is_ok(),
			"Case {}: Expected '{}' to normalize successfully, but got error: {:?}",
			i,
			input,
			result.err()
		);
		assert_eq!(
			result.unwrap(),
			*expected,
			"Case {}: Normalizing '{}' failed",
			i,
			input
		);
	}
}

#[test]
fn test_normalize_fails() {
	for (i, input) in NORMALIZE_FAIL_CASES.iter().enumerate() {
		let result = VersionParser::normalize(input);
		assert!(
			result.is_err(),
			"Case {}: Expected '{}' to fail normalization, but got: {:?}",
			i,
			input,
			result.ok()
		);
	}
}

#[test]
fn test_normalize_branch() {
	for (i, (input, expected)) in NORMALIZE_BRANCH_CASES.iter().enumerate() {
		let result = VersionParser::normalize_branch(input);
		assert_eq!(
			result, *expected,
			"Case {}: normalize_branch('{}') failed",
			i, input
		);
	}
}

#[test]
fn test_parse_numeric_alias_prefix() {
	for (i, (input, expected)) in NUMERIC_ALIAS_PREFIX_CASES.iter().enumerate() {
		let result = VersionParser::parse_numeric_alias_prefix(input);
		match expected {
			Some(exp) => {
				assert_eq!(
					result.as_deref(),
					Some(*exp),
					"Case {}: parse_numeric_alias_prefix('{}') failed",
					i,
					input
				);
			},
			None => {
				assert!(
					result.is_none(),
					"Case {}: Expected None for '{}', got {:?}",
					i,
					input,
					result
				);
			},
		}
	}
}

#[test]
fn test_is_valid() {
	for (i, (input, expected)) in IS_VALID_CASES.iter().enumerate() {
		let result = VersionParser::is_valid(input);
		assert_eq!(
			result, *expected,
			"Case {}: is_valid('{}') expected {}, got {}",
			i, input, expected, result
		);
	}
}

#[test]
fn test_parse_stability() {
	for (i, (input, expected)) in STABILITY_CASES.iter().enumerate() {
		let result = VersionParser::parse_stability(input);
		let expected_stability = match *expected {
			"stable" => Stability::Stable,
			"dev" => Stability::Dev,
			"alpha" => Stability::Alpha,
			"beta" => Stability::Beta,
			"RC" => Stability::RC,
			_ => panic!("Unknown stability: {}", expected),
		};
		assert_eq!(
			result, expected_stability,
			"Case {}: parse_stability('{}') expected {}, got {:?}",
			i, input, expected, result
		);
	}
}

#[test]
fn test_normalize_stability() {
	assert_eq!(VersionParser::normalize_stability("rc").unwrap(), "RC");
	assert_eq!(VersionParser::normalize_stability("RC").unwrap(), "RC");
	assert_eq!(VersionParser::normalize_stability("BeTa").unwrap(), "beta");
	assert_eq!(
		VersionParser::normalize_stability("stable").unwrap(),
		"stable"
	);
	assert!(VersionParser::normalize_stability("invalid").is_err());
}

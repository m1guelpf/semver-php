//! Version comparison utilities.
//!
//! Provides functions to compare two versions using Composer's semver semantics.

use crate::{
	constraint::{Operator, SingleConstraint},
	error::Result,
	parser::VersionParser,
};

/// Compare two versions using the specified operator.
///
/// # Arguments
/// * `version1` - First version string (will be normalized)
/// * `operator` - Comparison operator (">", ">=", "<", "<=", "==", "=", "!=", "<>")
/// * `version2` - Second version string (will be normalized)
///
/// # Returns
/// `true` if the comparison holds, `false` otherwise.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn compare(version1: &str, operator: &str, version2: &str) -> Result<bool> {
	let v1_normalized = VersionParser::normalize(version1)?;
	let v2_normalized = VersionParser::normalize(version2)?;

	let op = Operator::parse(operator)?;
	let constraint = SingleConstraint::new(op, &v2_normalized);
	let provider = SingleConstraint::new(Operator::Eq, &v1_normalized);

	Ok(constraint.match_specific(&provider, true))
}

/// Check if version1 > version2.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn greater_than(version1: &str, version2: &str) -> Result<bool> {
	compare(version1, ">", version2)
}

/// Check if version1 >= version2.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn greater_than_or_equal_to(version1: &str, version2: &str) -> Result<bool> {
	compare(version1, ">=", version2)
}

/// Check if version1 < version2.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn less_than(version1: &str, version2: &str) -> Result<bool> {
	compare(version1, "<", version2)
}

/// Check if version1 <= version2.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn less_than_or_equal_to(version1: &str, version2: &str) -> Result<bool> {
	compare(version1, "<=", version2)
}

/// Check if version1 == version2.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn equal_to(version1: &str, version2: &str) -> Result<bool> {
	compare(version1, "==", version2)
}

/// Check if version1 != version2.
///
/// # Errors
/// Returns an error if either version string is invalid.
pub fn not_equal_to(version1: &str, version2: &str) -> Result<bool> {
	compare(version1, "!=", version2)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_greater_than() {
		assert!(greater_than("1.25.0", "1.24.0").unwrap());
		assert!(!greater_than("1.25.0", "1.25.0").unwrap());
		assert!(!greater_than("1.25.0", "1.26.0").unwrap());
	}

	#[test]
	fn test_less_than() {
		assert!(!less_than("1.25.0", "1.24.0").unwrap());
		assert!(!less_than("1.25.0", "1.25.0").unwrap());
		assert!(less_than("1.25.0", "1.26.0").unwrap());
	}

	#[test]
	fn test_equal_to() {
		assert!(!equal_to("1.25.0", "1.24.0").unwrap());
		assert!(equal_to("1.25.0", "1.25.0").unwrap());
		assert!(!equal_to("1.25.0", "1.26.0").unwrap());
	}
}

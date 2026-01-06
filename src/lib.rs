#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../README.md")]

pub mod comparator;
pub mod constraint;
pub mod error;
pub mod interval;
pub mod intervals;
pub mod parser;
pub mod version;

pub use comparator::{
	compare, equal_to, greater_than, greater_than_or_equal_to, less_than, less_than_or_equal_to,
	not_equal_to,
};
pub use constraint::{
	Bound, Constraint, MatchAllConstraint, MatchNoneConstraint, MultiConstraint, Operator,
	SingleConstraint,
};
pub use error::{Result, SemverError};
pub use interval::{BranchConstraint, Interval, IntervalResult};
pub use intervals::Intervals;
pub use parser::VersionParser;
pub use version::{expand_stability, stability_order, version_compare, Stability};

/// Provides convenient methods for version comparison, constraint matching, and sorting.
pub struct Semver;

impl Semver {
	/// Check if a version satisfies a constraint string.
	///
	/// # Arguments
	/// * `version` - Version string to check
	/// * `constraints` - Constraint expression (e.g., "^1.0", ">=2.0 <3.0", "~1.2 || ^2.0")
	///
	/// # Errors
	/// Returns an error if the version or constraint string is invalid.
	///
	/// # Examples
	/// ```
	/// use semver_php::Semver;
	///
	/// assert!(Semver::satisfies("1.2.3", "^1.0").unwrap());
	/// assert!(Semver::satisfies("1.2.3", ">=1.0 <2.0").unwrap());
	/// assert!(!Semver::satisfies("2.0.0", "^1.0").unwrap());
	/// ```
	pub fn satisfies(version: &str, constraints: &str) -> Result<bool> {
		let normalized = VersionParser::normalize(version)?;
		let provider = SingleConstraint::new(Operator::Eq, &normalized);
		let parsed_constraints = VersionParser::parse_constraints(constraints)?;

		Ok(parsed_constraints.matches(&provider))
	}

	/// Filter a list of versions to those that satisfy a constraint.
	///
	/// # Arguments
	/// * `versions` - List of version strings
	/// * `constraints` - Constraint expression
	///
	/// # Errors
	/// Returns an error if any version or the constraint string is invalid.
	///
	/// # Examples
	/// ```
	/// use semver_php::Semver;
	///
	/// let versions = vec!["1.0", "1.2", "2.0", "3.0"];
	/// let satisfied = Semver::satisfied_by(&versions, "~1.0").unwrap();
	/// assert_eq!(satisfied, vec!["1.0", "1.2"]);
	/// ```
	pub fn satisfied_by(versions: &[&str], constraints: &str) -> Result<Vec<String>> {
		let mut result = Vec::new();
		for version in versions {
			if Self::satisfies(version, constraints)? {
				result.push((*version).to_string());
			}
		}
		Ok(result)
	}

	/// Sort versions in ascending order.
	///
	/// # Errors
	/// Returns an error if any version string is invalid.
	///
	/// # Examples
	/// ```
	/// use semver_php::Semver;
	///
	/// let versions = vec!["1.0", "0.1", "3.2.1", "2.4.0"];
	/// let sorted = Semver::sort(&versions).unwrap();
	/// assert_eq!(sorted, vec!["0.1", "1.0", "2.4.0", "3.2.1"]);
	/// ```
	pub fn sort(versions: &[&str]) -> Result<Vec<String>> {
		Self::usort(versions, true)
	}

	/// Sort versions in descending order.
	///
	/// # Errors
	/// Returns an error if any version string is invalid.
	///
	/// # Examples
	/// ```
	/// use semver_php::Semver;
	///
	/// let versions = vec!["1.0", "0.1", "3.2.1", "2.4.0"];
	/// let sorted = Semver::rsort(&versions).unwrap();
	/// assert_eq!(sorted, vec!["3.2.1", "2.4.0", "1.0", "0.1"]);
	/// ```
	pub fn rsort(versions: &[&str]) -> Result<Vec<String>> {
		Self::usort(versions, false)
	}

	fn usort(versions: &[&str], ascending: bool) -> Result<Vec<String>> {
		// Normalize all versions and store with original index
		let mut normalized: Vec<(String, usize)> = Vec::with_capacity(versions.len());
		for (idx, version) in versions.iter().enumerate() {
			let mut norm = VersionParser::normalize(version)?;
			#[allow(deprecated)]
			{
				norm = VersionParser::normalize_default_branch(&norm);
			}
			normalized.push((norm, idx));
		}

		// Sort by normalized version using version_compare directly
		// (avoids redundant normalization that less_than would do)
		normalized.sort_by(|(a, _), (b, _)| {
			let cmp = version_compare(a, b);
			if ascending {
				cmp
			} else {
				cmp.reverse()
			}
		});

		// Return original versions in sorted order
		Ok(normalized
			.into_iter()
			.map(|(_, idx)| versions[idx].to_string())
			.collect())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_constraint() {
		let c = SingleConstraint::new(Operator::Ge, "1.0.0.0");
		assert_eq!(c.operator(), Operator::Ge);
		assert_eq!(c.version(), "1.0.0.0");
	}

	#[test]
	fn test_match_all() {
		let all = MatchAllConstraint::new();
		let c = SingleConstraint::new(Operator::Eq, "1.0.0");
		assert!(all.matches(&c));
	}

	#[test]
	fn test_match_none() {
		let none = MatchNoneConstraint::new();
		let c = SingleConstraint::new(Operator::Eq, "1.0.0");
		assert!(!none.matches(&c));
	}
}

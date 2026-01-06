use crate::error::{Result, SemverError};
use std::{
	cmp::Ordering,
	fmt::{self, Display},
	mem,
};

/// Stability levels in Composer semver, ordered from least to most stable.
/// The ordering matters for comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stability {
	Dev,
	Alpha,
	Beta,
	RC,
	Stable,
}

impl Stability {
	/// Parse stability from string, handling short forms (a, b, p, pl, rc).
	/// Returns None if the string doesn't match a known stability.
	#[must_use]
	pub fn parse(s: &str) -> Option<Self> {
		match s.to_lowercase().as_str() {
			"rc" => Some(Self::RC),
			"beta" | "b" => Some(Self::Beta),
			"alpha" | "a" => Some(Self::Alpha),
			"dev" => Some(Self::Dev),
			"stable" | "patch" | "pl" | "p" => Some(Self::Stable), // patch is treated as stable
			_ => None,
		}
	}

	/// Normalize stability string to canonical form.
	///
	/// # Errors
	/// Returns `SemverError::InvalidStability` if the string is not a valid stability.
	pub fn normalize(s: &str) -> Result<Self> {
		let lower = s.to_lowercase();
		match lower.as_str() {
			"stable" | "rc" | "beta" | "alpha" | "dev" => {
				Self::parse(&lower).ok_or_else(|| SemverError::InvalidStability(s.to_string()))
			},
			_ => Err(SemverError::InvalidStability(s.to_string())),
		}
	}

	/// Get the canonical string representation.
	#[must_use]
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Dev => "dev",
			Self::Alpha => "alpha",
			Self::Beta => "beta",
			Self::RC => "RC",
			Self::Stable => "stable",
		}
	}

	/// Get the numeric order for comparison (lower = less stable).
	const fn order(self) -> i32 {
		match self {
			Self::Dev => -4,
			Self::Alpha => -3,
			Self::Beta => -2,
			Self::RC => -1,
			Self::Stable => 0,
		}
	}
}

impl PartialOrd for Stability {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Stability {
	fn cmp(&self, other: &Self) -> Ordering {
		(*self).order().cmp(&(*other).order())
	}
}

impl Display for Stability {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

/// Expand shorthand stability to full form.
/// E.g., "a" -> "alpha", "b" -> "beta", "p" -> "patch", "rc" -> "RC"
/// Returns a static string for known stabilities.
#[must_use]
pub fn expand_stability(stability: &str) -> &'static str {
	match stability.to_lowercase().as_str() {
		"a" | "alpha" => "alpha",
		"b" | "beta" => "beta",
		"p" | "pl" | "patch" => "patch",
		"rc" => "RC",
		"dev" => "dev",
		// Unknown stability defaults to stable
		_ => "stable",
	}
}

/// Get stability order for PHP `version_compare` semantics.
/// Returns numeric order where lower = less stable.
#[must_use]
pub fn stability_order(s: &str) -> i32 {
	match s.to_lowercase().as_str() {
		"dev" => -4,
		"alpha" | "a" => -3,
		"beta" | "b" => -2,
		"rc" => -1,
		"patch" | "pl" | "p" => 1,
		_ => 0, // stable/numeric
	}
}

/// Compare two version strings using PHP's `version_compare` semantics.
/// This is critical for matching Composer's behavior.
#[must_use]
pub fn version_compare(a: &str, b: &str) -> Ordering {
	let a_parts = split_version(a);
	let b_parts = split_version(b);

	let max_len = a_parts.len().max(b_parts.len());

	for i in 0..max_len {
		let a_part = a_parts.get(i).map_or("", String::as_str);
		let b_part = b_parts.get(i).map_or("", String::as_str);

		let cmp = compare_parts(a_part, b_part);
		if cmp != Ordering::Equal {
			return cmp;
		}
	}

	Ordering::Equal
}

/// Split version string into parts for comparison.
/// Splits on `.`, `-`, `_` characters.
fn split_version(version: &str) -> Vec<String> {
	let mut parts = Vec::new();
	let mut current = String::new();

	for ch in version.chars() {
		if ch == '.' || ch == '-' || ch == '_' {
			if !current.is_empty() {
				parts.push(mem::take(&mut current));
			}
		} else {
			current.push(ch);
		}
	}

	if !current.is_empty() {
		parts.push(current);
	}

	parts
}

/// Split a version part into stability prefix and numeric suffix.
/// E.g., "b2" -> ("beta", Some(2)), "beta2" -> ("beta", Some(2)), "15" -> ("", Some(15))
fn split_stability_and_number(s: &str) -> (&str, Option<i64>) {
	// Check for known stability prefixes with optional numeric suffix
	let lower = s.to_lowercase();

	// Try to match stability prefixes
	for (prefix, canonical) in &[
		("alpha", "alpha"),
		("beta", "beta"),
		("patch", "patch"),
		("dev", "dev"),
		("rc", "RC"),
		("pl", "patch"),
		("a", "alpha"),
		("b", "beta"),
		("p", "patch"),
	] {
		if lower.starts_with(prefix) {
			let rest = &s[prefix.len()..];
			// Remove leading dots or hyphens from the number
			let rest = rest.trim_start_matches(['.', '-']);
			let num = rest.parse::<i64>().ok();
			return (canonical, num);
		}
	}

	// Not a stability prefix - try to parse as number
	if let Ok(n) = s.parse::<i64>() {
		return ("", Some(n));
	}

	// Plain string
	(s, None)
}

/// Compare two version parts using PHP semantics.
fn compare_parts(a: &str, b: &str) -> Ordering {
	// Handle empty parts
	if a.is_empty() && b.is_empty() {
		return Ordering::Equal;
	}
	if a.is_empty() {
		// Empty is less than any non-empty, unless b is a stability marker
		let b_stability = stability_order(b);
		if b_stability != 0 {
			return Ordering::Greater; // "" > "dev", "" > "alpha", etc. but "" < "patch"
		}
		return Ordering::Less;
	}
	if b.is_empty() {
		let a_stability = stability_order(a);
		if a_stability != 0 {
			return Ordering::Less;
		}
		return Ordering::Greater;
	}

	// Split into stability prefix and number
	let (a_stability_str, a_num) = split_stability_and_number(a);
	let (b_stability_str, b_num) = split_stability_and_number(b);

	// If both have stability prefixes
	let a_stability_ord = stability_order(a_stability_str);
	let b_stability_ord = stability_order(b_stability_str);

	// First compare stability types
	if a_stability_ord != 0 || b_stability_ord != 0 {
		// At least one is a stability marker
		if a_stability_ord != b_stability_ord {
			return a_stability_ord.cmp(&b_stability_ord);
		}
		// Same stability type - compare numbers
		match (a_num, b_num) {
			(Some(an), Some(bn)) => return an.cmp(&bn),
			(Some(_), None) => return Ordering::Greater,
			(None, Some(_)) => return Ordering::Less,
			(None, None) => return Ordering::Equal,
		}
	}

	// Both are numeric or plain strings
	match (a_num, b_num) {
		(Some(an), Some(bn)) => an.cmp(&bn),
		(Some(_), None) => {
			// a is numeric, b is not
			Ordering::Greater
		},
		(None, Some(_)) => {
			// b is numeric, a is not
			Ordering::Less
		},
		(None, None) => {
			// Both are plain strings - compare lexicographically
			a.to_lowercase().cmp(&b.to_lowercase())
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_stability_ordering() {
		assert!(Stability::Dev < Stability::Alpha);
		assert!(Stability::Alpha < Stability::Beta);
		assert!(Stability::Beta < Stability::RC);
		assert!(Stability::RC < Stability::Stable);
	}

	#[test]
	fn test_stability_parse() {
		assert_eq!(Stability::parse("dev"), Some(Stability::Dev));
		assert_eq!(Stability::parse("alpha"), Some(Stability::Alpha));
		assert_eq!(Stability::parse("a"), Some(Stability::Alpha));
		assert_eq!(Stability::parse("beta"), Some(Stability::Beta));
		assert_eq!(Stability::parse("b"), Some(Stability::Beta));
		assert_eq!(Stability::parse("rc"), Some(Stability::RC));
		assert_eq!(Stability::parse("RC"), Some(Stability::RC));
		assert_eq!(Stability::parse("stable"), Some(Stability::Stable));
		assert_eq!(Stability::parse("patch"), Some(Stability::Stable));
		assert_eq!(Stability::parse("invalid"), None);
	}

	#[test]
	fn test_version_compare() {
		assert_eq!(version_compare("1.0.0", "1.0.0"), Ordering::Equal);
		assert_eq!(version_compare("1.0.0", "1.0.1"), Ordering::Less);
		assert_eq!(version_compare("1.0.1", "1.0.0"), Ordering::Greater);
		assert_eq!(version_compare("1.0", "1.0.0"), Ordering::Less);
		assert_eq!(version_compare("1.0.0", "1.0"), Ordering::Greater);
		assert_eq!(version_compare("1.0.0-dev", "1.0.0"), Ordering::Less);
		assert_eq!(version_compare("1.0.0-alpha", "1.0.0-beta"), Ordering::Less);
		assert_eq!(version_compare("1.0.0-RC", "1.0.0"), Ordering::Less);
	}

	#[test]
	fn test_version_compare_short_forms() {
		// Short stability forms should be equivalent to full forms
		assert_eq!(version_compare("2.0-b2", "2.0-beta2"), Ordering::Equal);
		assert_eq!(version_compare("2.0-a1", "2.0-alpha1"), Ordering::Equal);
		assert_eq!(version_compare("2.0-RC1", "2.0-rc1"), Ordering::Equal);
		assert_eq!(version_compare("3.0-b2", "3.0-beta2"), Ordering::Equal);

		// Beta comes after alpha
		assert_eq!(version_compare("2.0-alpha1", "2.0-beta1"), Ordering::Less);
		assert_eq!(version_compare("2.0-a1", "2.0-b1"), Ordering::Less);

		// Numeric suffixes matter
		assert_eq!(version_compare("2.0-beta1", "2.0-beta2"), Ordering::Less);
		assert_eq!(version_compare("2.0-b1", "2.0-b2"), Ordering::Less);
	}

	#[test]
	fn test_expand_stability() {
		assert_eq!(expand_stability("a"), "alpha");
		assert_eq!(expand_stability("b"), "beta");
		assert_eq!(expand_stability("p"), "patch");
		assert_eq!(expand_stability("pl"), "patch");
		assert_eq!(expand_stability("rc"), "RC");
		assert_eq!(expand_stability("alpha"), "alpha");
	}
}

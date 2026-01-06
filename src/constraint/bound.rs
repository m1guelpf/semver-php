use crate::version::version_compare;
use std::{
	cmp::Ordering,
	fmt::{self, Display},
};

/// Represents a version boundary (inclusive or exclusive).
/// Used to determine the range of versions a constraint covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
	version: String,
	inclusive: bool,
}

impl Bound {
	/// Create a new bound.
	pub fn new(version: impl Into<String>, inclusive: bool) -> Self {
		Self {
			version: version.into(),
			inclusive,
		}
	}

	/// The zero bound: 0.0.0.0-dev (inclusive).
	/// Represents the absolute minimum version.
	#[must_use]
	pub fn zero() -> Self {
		Self::new("0.0.0.0-dev", true)
	}

	/// Positive infinity bound (exclusive).
	/// PHP uses `PHP_INT_MAX` which is 9223372036854775807 on 64-bit.
	#[must_use]
	pub fn positive_infinity() -> Self {
		Self::new("9223372036854775807.0.0.0", false)
	}

	/// Get the version string.
	#[must_use]
	pub fn version(&self) -> &str {
		&self.version
	}

	/// Check if this bound is inclusive.
	#[must_use]
	pub const fn is_inclusive(&self) -> bool {
		self.inclusive
	}

	/// Check if this is the zero bound.
	#[must_use]
	pub fn is_zero(&self) -> bool {
		self.version == "0.0.0.0-dev" && self.inclusive
	}

	/// Check if this is the positive infinity bound.
	#[must_use]
	pub fn is_positive_infinity(&self) -> bool {
		self.version == "9223372036854775807.0.0.0" && !self.inclusive
	}

	/// Compare this bound to another.
	/// Returns ordering based on version comparison and inclusivity.
	#[must_use]
	pub fn compare_to(&self, other: &Self) -> Ordering {
		let cmp = version_compare(&self.version, &other.version);

		if cmp != Ordering::Equal {
			return cmp;
		}

		// If versions are equal, inclusivity matters
		// For lower bounds: inclusive < exclusive (inclusive starts earlier)
		// For upper bounds: exclusive < inclusive (exclusive ends earlier)
		// This comparison assumes lower bound context
		match (self.inclusive, other.inclusive) {
			(true, false) => Ordering::Less,
			(false, true) => Ordering::Greater,
			_ => Ordering::Equal,
		}
	}

	/// Compare for lower bound context.
	/// In lower bounds, inclusive is "smaller" (starts earlier).
	#[must_use]
	pub fn compare_as_lower(&self, other: &Self) -> Ordering {
		let cmp = version_compare(&self.version, &other.version);

		if cmp != Ordering::Equal {
			return cmp;
		}

		// For lower bounds: inclusive starts earlier than exclusive
		match (self.inclusive, other.inclusive) {
			(true, false) => Ordering::Less,
			(false, true) => Ordering::Greater,
			_ => Ordering::Equal,
		}
	}

	/// Compare for upper bound context.
	/// In upper bounds, exclusive is "smaller" (ends earlier).
	#[must_use]
	pub fn compare_as_upper(&self, other: &Self) -> Ordering {
		let cmp = version_compare(&self.version, &other.version);

		if cmp != Ordering::Equal {
			return cmp;
		}

		// For upper bounds: exclusive ends earlier than inclusive
		match (self.inclusive, other.inclusive) {
			(false, true) => Ordering::Less,
			(true, false) => Ordering::Greater,
			_ => Ordering::Equal,
		}
	}
}

impl Display for Bound {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let bracket = if self.inclusive {
			"inclusive"
		} else {
			"exclusive"
		};

		write!(f, "{} ({})", self.version, bracket)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bound_creation() {
		let bound = Bound::new("1.0.0", true);
		assert_eq!(bound.version(), "1.0.0");
		assert!(bound.is_inclusive());
	}

	#[test]
	fn test_zero_bound() {
		let zero = Bound::zero();
		assert!(zero.is_zero());
		assert!(!zero.is_positive_infinity());
	}

	#[test]
	fn test_positive_infinity() {
		let inf = Bound::positive_infinity();
		assert!(inf.is_positive_infinity());
		assert!(!inf.is_zero());
	}

	#[test]
	fn test_bound_comparison() {
		let a = Bound::new("1.0.0", true);
		let b = Bound::new("2.0.0", true);
		assert_eq!(a.compare_to(&b), Ordering::Less);

		let c = Bound::new("1.0.0", false);
		assert_eq!(a.compare_to(&c), Ordering::Less); // inclusive < exclusive for same version
	}
}

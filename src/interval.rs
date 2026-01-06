//! Single interval representation for numeric version ranges.

use std::fmt::{self, Display};

use crate::constraint::{Operator, SingleConstraint};

/// Represents a numeric version interval with start and end bounds.
#[derive(Debug, Clone)]
pub struct Interval {
	start: SingleConstraint,
	end: SingleConstraint,
}

impl Interval {
	/// Create a new interval from start and end constraints.
	#[must_use]
	pub const fn new(start: SingleConstraint, end: SingleConstraint) -> Self {
		Self { start, end }
	}

	/// Get the start constraint.
	#[must_use]
	pub const fn start(&self) -> &SingleConstraint {
		&self.start
	}

	/// Get the end constraint.
	#[must_use]
	pub const fn end(&self) -> &SingleConstraint {
		&self.end
	}

	/// Create a constraint representing "from zero" (>= 0.0.0.0-dev).
	#[must_use]
	pub fn from_zero() -> SingleConstraint {
		SingleConstraint::new(Operator::Ge, "0.0.0.0-dev")
	}

	/// Create a constraint representing "until positive infinity" (< MAX.0.0.0).
	#[must_use]
	pub fn until_positive_infinity() -> SingleConstraint {
		// Use a very large number like PHP's PHP_INT_MAX
		SingleConstraint::new(Operator::Lt, "9223372036854775807.0.0.0")
	}

	/// Create an interval covering everything (0 to +inf).
	#[must_use]
	pub fn any() -> Self {
		Self::new(Self::from_zero(), Self::until_positive_infinity())
	}
}

/// Branch constraints representation.
/// - `names`: list of branch names
/// - `exclude`: if true, all branches EXCEPT these are matched;
///   if false, ONLY these branches are matched
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchConstraint {
	pub names: Vec<String>,
	pub exclude: bool,
}

impl BranchConstraint {
	/// Create a new branch constraint.
	#[must_use]
	pub const fn new(names: Vec<String>, exclude: bool) -> Self {
		Self { names, exclude }
	}

	/// Match any dev branch (exclude nothing = match all).
	#[must_use]
	pub const fn any_dev() -> Self {
		Self {
			names: Vec::new(),
			exclude: true,
		}
	}

	/// Match no dev branches (include nothing).
	#[must_use]
	pub const fn no_dev() -> Self {
		Self {
			names: Vec::new(),
			exclude: false,
		}
	}

	/// Check if this matches any dev branch.
	#[must_use]
	pub const fn matches_any(&self) -> bool {
		self.exclude && self.names.is_empty()
	}

	/// Check if this matches no dev branches.
	#[must_use]
	pub const fn matches_none(&self) -> bool {
		!self.exclude && self.names.is_empty()
	}
}

impl Default for BranchConstraint {
	fn default() -> Self {
		Self::no_dev()
	}
}

/// Result of converting a constraint to intervals.
#[derive(Debug, Clone)]
pub struct IntervalResult {
	pub numeric: Vec<Interval>,
	pub branches: BranchConstraint,
}

impl IntervalResult {
	#[must_use]
	pub const fn new(numeric: Vec<Interval>, branches: BranchConstraint) -> Self {
		Self { numeric, branches }
	}

	/// Create result matching nothing.
	#[must_use]
	pub const fn none() -> Self {
		Self {
			numeric: Vec::new(),
			branches: BranchConstraint::no_dev(),
		}
	}

	/// Create result matching everything.
	#[must_use]
	pub fn any() -> Self {
		Self {
			numeric: vec![Interval::any()],
			branches: BranchConstraint::any_dev(),
		}
	}

	/// Check if this result matches anything.
	#[must_use]
	pub const fn matches_something(&self) -> bool {
		!self.numeric.is_empty() || self.branches.exclude || !self.branches.names.is_empty()
	}
}

impl Default for IntervalResult {
	fn default() -> Self {
		Self::none()
	}
}

impl Display for Interval {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[{} - {}]", self.start, self.end)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_interval_creation() {
		let interval = Interval::new(
			SingleConstraint::new(Operator::Ge, "1.0.0.0-dev"),
			SingleConstraint::new(Operator::Lt, "2.0.0.0-dev"),
		);
		assert_eq!(interval.start().operator(), Operator::Ge);
		assert_eq!(interval.end().operator(), Operator::Lt);
	}

	#[test]
	fn test_branch_constraint() {
		let any = BranchConstraint::any_dev();
		assert!(any.matches_any());
		assert!(!any.matches_none());

		let none = BranchConstraint::no_dev();
		assert!(none.matches_none());
		assert!(!none.matches_any());
	}
}

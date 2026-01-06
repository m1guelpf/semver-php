use super::{Bound, Constraint, Operator};
use crate::version::version_compare;
use std::{cmp::Ordering, fmt};

/// A single version constraint (e.g., >=1.0.0, <2.0.0, ==1.5.0).
#[derive(Debug, Clone)]
pub struct SingleConstraint {
	operator: Operator,
	version: String,
	pretty_string: Option<String>,
}

impl SingleConstraint {
	/// Create a new single constraint.
	pub fn new(operator: Operator, version: impl Into<String>) -> Self {
		Self {
			operator,
			version: version.into(),
			pretty_string: None,
		}
	}

	/// Get the operator.
	#[must_use]
	pub const fn operator(&self) -> Operator {
		self.operator
	}

	/// Get the version string.
	#[must_use]
	pub fn version(&self) -> &str {
		&self.version
	}

	/// Check if this version is a dev branch (starts with "dev-").
	#[must_use]
	pub fn is_dev_branch(&self) -> bool {
		self.version.starts_with("dev-")
	}

	/// Match against another single constraint.
	/// This implements the core matching logic from PHP's `Constraint::matchSpecific`.
	#[must_use]
	pub fn match_specific(&self, other: &Self, compare_branches: bool) -> bool {
		let self_is_branch = self.is_dev_branch();
		let other_is_branch = other.is_dev_branch();

		// Handle != operator specially
		let is_ne = self.operator == Operator::Ne;
		let is_other_ne = other.operator == Operator::Ne;

		if is_ne || is_other_ne {
			// Special case: != with dev branches
			// If one side is != and other is not (== or !=) and the other is a dev branch: no match
			if is_ne && !is_other_ne && other.operator != Operator::Eq && other_is_branch {
				return false;
			}
			if is_other_ne && !is_ne && self.operator != Operator::Eq && self_is_branch {
				return false;
			}

			// If neither is ==, there's always a solution for !=
			if self.operator != Operator::Eq && other.operator != Operator::Eq {
				return true;
			}

			// Otherwise compare versions (using version_compare for non-branches)
			if self_is_branch || other_is_branch {
				return self.version != other.version;
			}
			// Fall through to normal comparison
		}

		// Dev branches have special matching rules
		if self_is_branch && other_is_branch {
			// Both branches with == - match if same branch
			if self.operator == Operator::Eq && other.operator == Operator::Eq {
				return self.version == other.version;
			}

			// Any other combination with both branches doesn't match
			return false;
		}

		// If one is a branch and the other isn't
		if self_is_branch != other_is_branch {
			// When branches are not comparable, dev branches never match anything
			if !compare_branches {
				return false;
			}

			// When compare_branches is true, use version_compare
			// Fall through to normal comparison logic below
		}

		// Non-branch version comparisons

		// Handle != operator specially
		if self.operator == Operator::Ne {
			// != matches anything except == to the same version
			if other.operator == Operator::Eq {
				return self.version != other.version;
			}
			return true;
		}
		if other.operator == Operator::Ne {
			if self.operator == Operator::Eq {
				return self.version != other.version;
			}
			return true;
		}

		// For comparison operators
		let cmp = version_compare(&self.version, &other.version);

		// Check if constraints can have any intersection
		match (&self.operator, &other.operator) {
			// Both equality - must be same version
			(Operator::Eq, Operator::Eq) => cmp == Ordering::Equal,

			// One side is equality - check if it satisfies the other constraint
			(Operator::Eq, op) | (op, Operator::Eq) => {
				let (eq_version, other_op, other_version) = if self.operator == Operator::Eq {
					(&self.version, op, &other.version)
				} else {
					(&other.version, op, &self.version)
				};

				let cmp = version_compare(eq_version, other_version);
				match other_op {
					Operator::Lt => cmp == Ordering::Less,
					Operator::Le => cmp != Ordering::Greater,
					Operator::Gt => cmp == Ordering::Greater,
					Operator::Ge => cmp != Ordering::Less,
					Operator::Eq => cmp == Ordering::Equal,
					Operator::Ne => cmp != Ordering::Equal,
				}
			},

			// Both are range operators - check for intersection
			// Any < or <= constraints always intersect, as do any > or >= constraints
			(Operator::Lt | Operator::Le, Operator::Lt | Operator::Le)
			| (Operator::Gt | Operator::Ge, Operator::Gt | Operator::Ge) => true,

			// One upper bound, one lower bound - check for overlap
			(Operator::Lt | Operator::Le, Operator::Gt) | (Operator::Lt, Operator::Ge) => {
				cmp == Ordering::Greater
			},
			(Operator::Le, Operator::Ge) => cmp != Ordering::Less,

			(Operator::Gt | Operator::Ge, Operator::Lt) | (Operator::Gt, Operator::Le) => {
				cmp == Ordering::Less
			},
			(Operator::Ge, Operator::Le) => cmp != Ordering::Greater,

			_ => false,
		}
	}

	/// Extract the lower bound from this constraint.
	fn extract_lower_bound(&self) -> Bound {
		match self.operator {
			Operator::Ne | Operator::Lt | Operator::Le => Bound::zero(),
			Operator::Gt => Bound::new(&self.version, false),
			Operator::Eq | Operator::Ge => Bound::new(&self.version, true),
		}
	}

	/// Extract the upper bound from this constraint.
	fn extract_upper_bound(&self) -> Bound {
		match self.operator {
			Operator::Ne | Operator::Gt | Operator::Ge => Bound::positive_infinity(),
			Operator::Lt => Bound::new(&self.version, false),
			Operator::Eq | Operator::Le => Bound::new(&self.version, true),
		}
	}
}

impl Constraint for SingleConstraint {
	fn matches(&self, other: &dyn Constraint) -> bool {
		// Try to match against specific types
		if let Some(single) = other.as_single() {
			return self.match_specific(single, false);
		}

		if other.is_match_all() {
			return true;
		}

		if other.is_match_none() {
			return false;
		}

		// For multi-constraints, delegate to their matching logic
		if let Some(multi) = other.as_multi() {
			return multi.matches(self);
		}

		// Default: use bounds-based matching
		// This is a fallback and shouldn't normally be hit
		true
	}

	fn lower_bound(&self) -> Bound {
		self.extract_lower_bound()
	}

	fn upper_bound(&self) -> Bound {
		self.extract_upper_bound()
	}

	fn set_pretty_string(&mut self, pretty: String) {
		self.pretty_string = Some(pretty);
	}

	fn pretty_string(&self) -> String {
		self.pretty_string
			.clone()
			.unwrap_or_else(|| self.to_string())
	}

	fn as_single(&self) -> Option<&SingleConstraint> {
		Some(self)
	}
}

impl fmt::Display for SingleConstraint {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Use == for display instead of =
		let op = if self.operator == Operator::Eq {
			"=="
		} else {
			self.operator.as_str()
		};
		write!(f, "{} {}", op, self.version)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_single_constraint_creation() {
		let c = SingleConstraint::new(Operator::Ge, "1.0.0");
		assert_eq!(c.operator(), Operator::Ge);
		assert_eq!(c.version(), "1.0.0");
		assert!(!c.is_dev_branch());
	}

	#[test]
	fn test_dev_branch_detection() {
		let c = SingleConstraint::new(Operator::Eq, "dev-master");
		assert!(c.is_dev_branch());

		let c2 = SingleConstraint::new(Operator::Eq, "1.0.0");
		assert!(!c2.is_dev_branch());
	}

	#[test]
	fn test_basic_matching() {
		let eq_2 = SingleConstraint::new(Operator::Eq, "2.0.0.0");
		let lt_3 = SingleConstraint::new(Operator::Lt, "3.0.0.0");
		assert!(eq_2.match_specific(&lt_3, false));

		let gt_3 = SingleConstraint::new(Operator::Gt, "3.0.0.0");
		assert!(!eq_2.match_specific(&gt_3, false));
	}

	#[test]
	fn test_bounds() {
		let c = SingleConstraint::new(Operator::Ge, "1.0.0");
		let lower = c.lower_bound();
		assert_eq!(lower.version(), "1.0.0");
		assert!(lower.is_inclusive());

		let upper = c.upper_bound();
		assert!(upper.is_positive_infinity());
	}
}

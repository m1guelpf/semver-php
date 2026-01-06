use super::{Bound, Constraint};
use std::fmt;

/// A constraint that matches all versions (wildcard *).
#[derive(Debug, Clone, Default)]
pub struct MatchAllConstraint {
	pretty_string: Option<String>,
}

impl MatchAllConstraint {
	/// Create a new match-all constraint.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}
}

impl Constraint for MatchAllConstraint {
	fn matches(&self, _other: &dyn Constraint) -> bool {
		// Match-all always returns true
		true
	}

	fn lower_bound(&self) -> Bound {
		Bound::zero()
	}

	fn upper_bound(&self) -> Bound {
		Bound::positive_infinity()
	}

	fn is_match_all(&self) -> bool {
		true
	}

	fn set_pretty_string(&mut self, pretty: String) {
		self.pretty_string = Some(pretty);
	}

	fn pretty_string(&self) -> String {
		self.pretty_string
			.clone()
			.unwrap_or_else(|| self.to_string())
	}
}

impl fmt::Display for MatchAllConstraint {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "*")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::constraint::{Operator, SingleConstraint};

	#[test]
	fn test_match_all_matches_everything() {
		let match_all = MatchAllConstraint::new();

		let eq = SingleConstraint::new(Operator::Eq, "1.0.0");
		assert!(match_all.matches(&eq));

		let lt = SingleConstraint::new(Operator::Lt, "2.0.0");
		assert!(match_all.matches(&lt));
	}

	#[test]
	fn test_match_all_bounds() {
		let match_all = MatchAllConstraint::new();
		assert!(match_all.lower_bound().is_zero());
		assert!(match_all.upper_bound().is_positive_infinity());
	}

	#[test]
	fn test_is_match_all() {
		let match_all = MatchAllConstraint::new();
		assert!(match_all.is_match_all());
		assert!(!match_all.is_match_none());
	}
}

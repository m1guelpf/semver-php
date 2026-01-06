use super::{Bound, Constraint};
use std::fmt;

/// A constraint that matches no versions (empty/impossible constraint).
#[derive(Debug, Clone, Default)]
pub struct MatchNoneConstraint {
	pretty_string: Option<String>,
}

impl MatchNoneConstraint {
	/// Create a new match-none constraint.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}
}

impl Constraint for MatchNoneConstraint {
	fn matches(&self, _other: &dyn Constraint) -> bool {
		// Match-none always returns false
		false
	}

	fn lower_bound(&self) -> Bound {
		// Empty range: exclusive zero
		Bound::new("0.0.0.0-dev", false)
	}

	fn upper_bound(&self) -> Bound {
		// Empty range: exclusive zero
		Bound::new("0.0.0.0-dev", false)
	}

	fn is_match_none(&self) -> bool {
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

impl fmt::Display for MatchNoneConstraint {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[]")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::constraint::{Operator, SingleConstraint};

	#[test]
	fn test_match_none_matches_nothing() {
		let match_none = MatchNoneConstraint::new();

		let eq = SingleConstraint::new(Operator::Eq, "1.0.0");
		assert!(!match_none.matches(&eq));

		let lt = SingleConstraint::new(Operator::Lt, "2.0.0");
		assert!(!match_none.matches(&lt));
	}

	#[test]
	fn test_is_match_none() {
		let match_none = MatchNoneConstraint::new();
		assert!(match_none.is_match_none());
		assert!(!match_none.is_match_all());
	}
}

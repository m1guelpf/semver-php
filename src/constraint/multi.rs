use super::{Bound, Constraint, MatchAllConstraint, MatchNoneConstraint};
use std::fmt;

/// A compound constraint combining multiple constraints with AND or OR.
#[derive(Debug)]
pub struct MultiConstraint {
	constraints: Vec<Box<dyn Constraint>>,
	conjunctive: bool, // true = AND, false = OR
	pretty_string: Option<String>,
}

impl MultiConstraint {
	/// Create a new multi-constraint.
	/// `conjunctive` = true means AND (all must match), false means OR (any must match).
	#[must_use]
	pub fn new(constraints: Vec<Box<dyn Constraint>>, conjunctive: bool) -> Self {
		Self {
			constraints,
			conjunctive,
			pretty_string: None,
		}
	}

	/// Smart constructor that optimizes and handles edge cases.
	/// Returns appropriate type: `MatchAll`, `MatchNone`, Single, or Multi.
	///
	/// # Panics
	/// Never panics - the unwrap is only reached when length is exactly 1.
	#[must_use]
	pub fn create(constraints: Vec<Box<dyn Constraint>>, conjunctive: bool) -> Box<dyn Constraint> {
		let mut filtered: Vec<Box<dyn Constraint>> = Vec::new();

		for c in constraints {
			// Skip match-all in conjunctive (AND), skip match-none in disjunctive (OR)
			if conjunctive && c.is_match_all() {
				continue;
			}
			if !conjunctive && c.is_match_none() {
				continue;
			}

			// Short-circuit: match-none in conjunctive = match-none
			if conjunctive && c.is_match_none() {
				return Box::new(MatchNoneConstraint::new());
			}
			// Short-circuit: match-all in disjunctive = match-all
			if !conjunctive && c.is_match_all() {
				return Box::new(MatchAllConstraint::new());
			}

			// Flatten nested multi-constraints of the same type
			// TODO: implement proper cloning/flattening of nested multi-constraints
			if let Some(multi) = c.as_multi() {
				if multi.conjunctive == conjunctive {
					// Would need to clone inner constraints here
					// For now, just add the whole multi
				}
			}

			filtered.push(c);
		}

		match filtered.len() {
			0 => {
				if conjunctive {
					Box::new(MatchAllConstraint::new())
				} else {
					Box::new(MatchNoneConstraint::new())
				}
			},
			1 => filtered.into_iter().next().unwrap(),
			_ => Box::new(Self::new(filtered, conjunctive)),
		}
	}

	/// Check if this is a conjunctive (AND) constraint.
	#[must_use]
	pub const fn is_conjunctive(&self) -> bool {
		self.conjunctive
	}

	/// Check if this is a disjunctive (OR) constraint.
	#[must_use]
	pub const fn is_disjunctive(&self) -> bool {
		!self.conjunctive
	}

	/// Get the inner constraints.
	#[must_use]
	pub fn constraints(&self) -> &[Box<dyn Constraint>] {
		&self.constraints
	}
}

impl Constraint for MultiConstraint {
	fn matches(&self, other: &dyn Constraint) -> bool {
		if other.is_match_none() {
			return false;
		}

		if other.is_match_all() {
			return true;
		}

		// For disjunctive multi-constraints as "other", we need special handling
		if let Some(other_multi) = other.as_multi() {
			if other_multi.is_disjunctive() {
				// For disjunctive other, any of their constraints matching is enough
				for other_c in &other_multi.constraints {
					if self.matches(other_c.as_ref()) {
						return true;
					}
				}
				return false;
			}
		}

		if self.conjunctive {
			// AND: all constraints must match
			self.constraints.iter().all(|c| c.matches(other))
		} else {
			// OR: at least one constraint must match
			self.constraints.iter().any(|c| c.matches(other))
		}
	}

	fn lower_bound(&self) -> Bound {
		if self.constraints.is_empty() {
			return Bound::zero();
		}

		if self.conjunctive {
			// AND: take the highest (most restrictive) lower bound
			self.constraints
				.iter()
				.map(|c| c.lower_bound())
				.max_by(Bound::compare_as_lower)
				.unwrap_or_else(Bound::zero)
		} else {
			// OR: take the lowest (least restrictive) lower bound
			self.constraints
				.iter()
				.map(|c| c.lower_bound())
				.min_by(Bound::compare_as_lower)
				.unwrap_or_else(Bound::zero)
		}
	}

	fn upper_bound(&self) -> Bound {
		if self.constraints.is_empty() {
			return Bound::positive_infinity();
		}

		if self.conjunctive {
			// AND: take the lowest (most restrictive) upper bound
			self.constraints
				.iter()
				.map(|c| c.upper_bound())
				.min_by(Bound::compare_as_upper)
				.unwrap_or_else(Bound::positive_infinity)
		} else {
			// OR: take the highest (least restrictive) upper bound
			self.constraints
				.iter()
				.map(|c| c.upper_bound())
				.max_by(Bound::compare_as_upper)
				.unwrap_or_else(Bound::positive_infinity)
		}
	}

	fn set_pretty_string(&mut self, pretty: String) {
		self.pretty_string = Some(pretty);
	}

	fn pretty_string(&self) -> String {
		self.pretty_string
			.clone()
			.unwrap_or_else(|| self.to_string())
	}

	fn as_multi(&self) -> Option<&MultiConstraint> {
		Some(self)
	}
}

impl fmt::Display for MultiConstraint {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let sep = if self.conjunctive { " " } else { " || " };
		let parts: Vec<String> = self.constraints.iter().map(ToString::to_string).collect();
		write!(f, "[{}]", parts.join(sep))
	}
}

impl Clone for MultiConstraint {
	fn clone(&self) -> Self {
		// We need to clone the boxed constraints
		// For now, we'll create new boxes with the display representation
		// This is a limitation - proper cloning would require Clone on dyn Constraint
		Self {
			constraints: Vec::new(), // TODO: implement proper cloning
			conjunctive: self.conjunctive,
			pretty_string: self.pretty_string.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::constraint::{Operator, SingleConstraint};

	#[test]
	fn test_conjunctive_matching() {
		let c1 = Box::new(SingleConstraint::new(Operator::Gt, "1.0.0.0"));
		let c2 = Box::new(SingleConstraint::new(Operator::Lt, "2.0.0.0"));
		let multi = MultiConstraint::new(vec![c1, c2], true);

		// 1.5.0 should match (> 1.0 AND < 2.0)
		let v = SingleConstraint::new(Operator::Eq, "1.5.0.0");
		assert!(multi.matches(&v));

		// 0.5.0 should not match (not > 1.0)
		let v2 = SingleConstraint::new(Operator::Eq, "0.5.0.0");
		assert!(!multi.matches(&v2));
	}

	#[test]
	fn test_disjunctive_matching() {
		let c1 = Box::new(SingleConstraint::new(Operator::Lt, "1.0.0.0"));
		let c2 = Box::new(SingleConstraint::new(Operator::Gt, "2.0.0.0"));
		let multi = MultiConstraint::new(vec![c1, c2], false);

		// 0.5.0 should match (< 1.0)
		let v = SingleConstraint::new(Operator::Eq, "0.5.0.0");
		assert!(multi.matches(&v));

		// 3.0.0 should match (> 2.0)
		let v2 = SingleConstraint::new(Operator::Eq, "3.0.0.0");
		assert!(multi.matches(&v2));

		// 1.5.0 should not match (not < 1.0 and not > 2.0)
		let v3 = SingleConstraint::new(Operator::Eq, "1.5.0.0");
		assert!(!multi.matches(&v3));
	}

	#[test]
	fn test_create_optimization() {
		// Empty conjunctive -> MatchAll
		let result = MultiConstraint::create(vec![], true);
		assert!(result.is_match_all());

		// Empty disjunctive -> MatchNone
		let result = MultiConstraint::create(vec![], false);
		assert!(result.is_match_none());

		// Single constraint -> unwrap
		let c = Box::new(SingleConstraint::new(Operator::Eq, "1.0.0"));
		let result = MultiConstraint::create(vec![c], true);
		assert!(result.as_single().is_some());
	}
}

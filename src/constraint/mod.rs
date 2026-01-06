pub mod bound;
pub mod match_all;
pub mod match_none;
pub mod multi;
pub mod single;

pub use bound::Bound;
pub use match_all::MatchAllConstraint;
pub use match_none::MatchNoneConstraint;
pub use multi::MultiConstraint;
pub use single::SingleConstraint;

use crate::error::{Result, SemverError};
use std::fmt;

/// Comparison operators for version constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
	/// Equal (== or =)
	Eq,
	/// Not equal (!= or <>)
	Ne,
	/// Less than (<)
	Lt,
	/// Less than or equal (<=)
	Le,
	/// Greater than (>)
	Gt,
	/// Greater than or equal (>=)
	Ge,
}

impl Operator {
	/// Parse an operator from a string.
	///
	/// # Errors
	/// Returns `SemverError::InvalidOperator` if the string is not a recognized operator.
	pub fn parse(s: &str) -> Result<Self> {
		match s {
			"=" | "==" => Ok(Self::Eq),
			"!=" | "<>" => Ok(Self::Ne),
			"<" => Ok(Self::Lt),
			"<=" => Ok(Self::Le),
			">" => Ok(Self::Gt),
			">=" => Ok(Self::Ge),
			_ => Err(SemverError::InvalidOperator(s.to_string())),
		}
	}

	/// Get the string representation of the operator.
	#[must_use]
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Eq => "==",
			Self::Ne => "!=",
			Self::Lt => "<",
			Self::Le => "<=",
			Self::Gt => ">",
			Self::Ge => ">=",
		}
	}

	/// Get the inverse operator (for matching logic).
	#[must_use]
	pub const fn inverse(&self) -> Self {
		match self {
			Self::Eq => Self::Eq,
			Self::Ne => Self::Ne,
			Self::Lt => Self::Gt,
			Self::Le => Self::Ge,
			Self::Gt => Self::Lt,
			Self::Ge => Self::Le,
		}
	}
}

impl fmt::Display for Operator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

/// Core trait for all constraint types.
pub trait Constraint: fmt::Display + fmt::Debug + Send + Sync {
	/// Check if this constraint matches/intersects with another.
	fn matches(&self, other: &dyn Constraint) -> bool;

	/// Get the lower bound of this constraint.
	fn lower_bound(&self) -> Bound;

	/// Get the upper bound of this constraint.
	fn upper_bound(&self) -> Bound;

	/// Get a human-readable representation.
	fn pretty_string(&self) -> String {
		self.to_string()
	}

	/// Set a custom pretty string (for preserving user input).
	fn set_pretty_string(&mut self, _pretty: String) {
		// Default: do nothing
	}

	/// Check if this is a `MatchAllConstraint`.
	fn is_match_all(&self) -> bool {
		false
	}

	/// Check if this is a `MatchNoneConstraint`.
	fn is_match_none(&self) -> bool {
		false
	}

	/// Try to downcast to `SingleConstraint`.
	fn as_single(&self) -> Option<&SingleConstraint> {
		None
	}

	/// Try to downcast to `MultiConstraint`.
	fn as_multi(&self) -> Option<&MultiConstraint> {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_operator_parse() {
		assert_eq!(Operator::parse("=").unwrap(), Operator::Eq);
		assert_eq!(Operator::parse("==").unwrap(), Operator::Eq);
		assert_eq!(Operator::parse("!=").unwrap(), Operator::Ne);
		assert_eq!(Operator::parse("<>").unwrap(), Operator::Ne);
		assert_eq!(Operator::parse("<").unwrap(), Operator::Lt);
		assert_eq!(Operator::parse("<=").unwrap(), Operator::Le);
		assert_eq!(Operator::parse(">").unwrap(), Operator::Gt);
		assert_eq!(Operator::parse(">=").unwrap(), Operator::Ge);
		assert!(Operator::parse("??").is_err());
	}

	#[test]
	fn test_operator_inverse() {
		assert_eq!(Operator::Lt.inverse(), Operator::Gt);
		assert_eq!(Operator::Gt.inverse(), Operator::Lt);
		assert_eq!(Operator::Le.inverse(), Operator::Ge);
		assert_eq!(Operator::Ge.inverse(), Operator::Le);
		assert_eq!(Operator::Eq.inverse(), Operator::Eq);
		assert_eq!(Operator::Ne.inverse(), Operator::Ne);
	}
}

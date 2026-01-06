//! Interval algebra for constraints.
//!
//! Provides utilities for:
//! - Converting constraints to intervals
//! - Checking if one constraint is a subset of another
//! - Checking if two constraints have any intersection
//! - Compacting constraints

use crate::{
	constraint::{Constraint, MultiConstraint, Operator, SingleConstraint},
	interval::{BranchConstraint, Interval, IntervalResult},
	version::version_compare,
};
use std::{cmp::Ordering, collections::HashMap};

/// Intervals helper for constraint algebra.
pub struct Intervals {
	cache: HashMap<String, IntervalResult>,
}

impl Default for Intervals {
	fn default() -> Self {
		Self::new()
	}
}

impl Intervals {
	/// Create a new Intervals helper.
	#[must_use]
	pub fn new() -> Self {
		Self {
			cache: HashMap::new(),
		}
	}

	/// Clear the memoization cache.
	pub fn clear(&mut self) {
		self.cache.clear();
	}

	/// Check if `candidate` is a subset of `constraint`.
	pub fn is_subset_of(
		&mut self,
		candidate: &dyn Constraint,
		constraint: &dyn Constraint,
	) -> bool {
		// MatchAll contains everything
		if constraint.is_match_all() {
			return true;
		}

		// MatchNone contains nothing, and nothing is a subset of MatchNone except MatchNone itself
		if candidate.is_match_none() || constraint.is_match_none() {
			return false;
		}

		// Create intersection: candidate AND constraint
		let intersection = self.get_intersection(candidate, constraint);
		let candidate_intervals = self.get(candidate);

		// Check numeric intervals match
		if intersection.numeric.len() != candidate_intervals.numeric.len() {
			return false;
		}

		for (i, interval) in intersection.numeric.iter().enumerate() {
			if i >= candidate_intervals.numeric.len() {
				return false;
			}

			let cand_interval = &candidate_intervals.numeric[i];

			// Compare start bounds
			if interval.start().to_string() != cand_interval.start().to_string() {
				return false;
			}

			// Compare end bounds
			if interval.end().to_string() != cand_interval.end().to_string() {
				return false;
			}
		}

		// Check branch constraints match
		if intersection.branches.exclude != candidate_intervals.branches.exclude {
			return false;
		}

		if intersection.branches.names.len() != candidate_intervals.branches.names.len() {
			return false;
		}

		for (i, name) in intersection.branches.names.iter().enumerate() {
			if i >= candidate_intervals.branches.names.len()
				|| name != &candidate_intervals.branches.names[i]
			{
				return false;
			}
		}

		true
	}

	/// Check if two constraints have any intersection.
	pub fn have_intersections(&mut self, a: &dyn Constraint, b: &dyn Constraint) -> bool {
		if a.is_match_all() || b.is_match_all() {
			return true;
		}

		if a.is_match_none() || b.is_match_none() {
			return false;
		}

		let intersection = self.get_intersection_early_exit(a, b);
		intersection.matches_something()
	}

	/// Get intervals for a constraint (with caching).
	pub fn get(&mut self, constraint: &dyn Constraint) -> IntervalResult {
		let key = constraint.to_string();

		if let Some(cached) = self.cache.get(&key) {
			return cached.clone();
		}

		let result = self.generate_intervals(constraint, false);
		self.cache.insert(key, result.clone());
		result
	}

	/// Get intersection of two constraints.
	fn get_intersection(&mut self, a: &dyn Constraint, b: &dyn Constraint) -> IntervalResult {
		// Create a conjunctive constraint and generate intervals
		let a_intervals = self.get(a);
		let b_intervals = self.get(b);
		self.intersect_intervals(&a_intervals, &b_intervals)
	}

	/// Get intersection with early exit on first valid interval.
	fn get_intersection_early_exit(
		&mut self,
		a: &dyn Constraint,
		b: &dyn Constraint,
	) -> IntervalResult {
		let a_intervals = self.get(a);
		let b_intervals = self.get(b);
		self.intersect_intervals_early_exit(&a_intervals, &b_intervals)
	}

	/// Generate intervals from a constraint.
	fn generate_intervals(
		&mut self,
		constraint: &dyn Constraint,
		stop_on_first: bool,
	) -> IntervalResult {
		if constraint.is_match_all() {
			return IntervalResult::any();
		}

		if constraint.is_match_none() {
			return IntervalResult::none();
		}

		if let Some(single) = constraint.as_single() {
			return self.generate_single_intervals(single);
		}

		if let Some(multi) = constraint.as_multi() {
			return self.generate_multi_intervals(multi, stop_on_first);
		}

		IntervalResult::none()
	}

	/// Generate intervals from a single constraint.
	#[allow(clippy::unused_self)]
	fn generate_single_intervals(&self, constraint: &SingleConstraint) -> IntervalResult {
		let op = constraint.operator();
		let version = constraint.version();

		// Handle dev branches
		if version.starts_with("dev-") {
			let mut intervals = Vec::new();
			let mut branches = BranchConstraint::no_dev();

			match op {
				Operator::Ne => {
					// != dev-foo means any numeric version may match
					intervals.push(Interval::any());
					branches = BranchConstraint::new(vec![version.to_string()], true);
				},
				Operator::Eq => {
					branches.names.push(version.to_string());
				},
				_ => {
					// Other operators on dev branches don't make sense for numeric ranges
				},
			}

			return IntervalResult::new(intervals, branches);
		}

		// Numeric version handling
		match op {
			Operator::Gt | Operator::Ge => IntervalResult::new(
				vec![Interval::new(
					SingleConstraint::new(op, version),
					Interval::until_positive_infinity(),
				)],
				BranchConstraint::no_dev(),
			),
			Operator::Lt | Operator::Le => IntervalResult::new(
				vec![Interval::new(
					Interval::from_zero(),
					SingleConstraint::new(op, version),
				)],
				BranchConstraint::no_dev(),
			),
			Operator::Ne => {
				// != x becomes: [0, <x) and (>x, +inf)
				IntervalResult::new(
					vec![
						Interval::new(
							Interval::from_zero(),
							SingleConstraint::new(Operator::Lt, version),
						),
						Interval::new(
							SingleConstraint::new(Operator::Gt, version),
							Interval::until_positive_infinity(),
						),
					],
					BranchConstraint::any_dev(),
				)
			},
			Operator::Eq => {
				// == x becomes: [>=x, <=x]
				IntervalResult::new(
					vec![Interval::new(
						SingleConstraint::new(Operator::Ge, version),
						SingleConstraint::new(Operator::Le, version),
					)],
					BranchConstraint::no_dev(),
				)
			},
		}
	}

	/// Generate intervals from a multi constraint.
	fn generate_multi_intervals(
		&mut self,
		multi: &MultiConstraint,
		stop_on_first: bool,
	) -> IntervalResult {
		let constraints = multi.constraints();

		let mut numeric_groups: Vec<Vec<Interval>> = Vec::new();
		let mut constraint_branches: Vec<BranchConstraint> = Vec::new();

		for c in constraints {
			let res = self.get(c.as_ref());
			numeric_groups.push(res.numeric);
			constraint_branches.push(res.branches);
		}

		// Compute branch result
		let branches = if multi.is_disjunctive() {
			self.merge_branches_disjunctive(&constraint_branches)
		} else {
			self.merge_branches_conjunctive(&constraint_branches)
		};

		// If only one group, return it
		if numeric_groups.len() == 1 {
			return IntervalResult::new(numeric_groups.into_iter().next().unwrap(), branches);
		}

		// Merge numeric intervals
		let numeric = if multi.is_disjunctive() {
			self.merge_intervals_disjunctive(&numeric_groups)
		} else {
			self.merge_intervals_conjunctive(&numeric_groups, stop_on_first)
		};

		IntervalResult::new(numeric, branches)
	}

	/// Merge branches for disjunctive constraints.
	#[allow(clippy::unused_self)]
	fn merge_branches_disjunctive(&self, branches: &[BranchConstraint]) -> BranchConstraint {
		let mut result = BranchConstraint::no_dev();

		for b in branches {
			if b.exclude {
				if result.exclude {
					// Disjunctive: exclude only what's excluded in all
					result.names.retain(|n| b.names.contains(n));
				} else {
					// Switch to exclude mode
					result.exclude = true;
					result.names = b
						.names
						.iter()
						.filter(|n| !result.names.contains(*n))
						.cloned()
						.collect();
				}
			} else if result.exclude {
				// Keep excluding, but remove any explicitly included
				result.names.retain(|n| !b.names.contains(n));
			} else {
				// Just add all branch names
				result.names.extend(b.names.iter().cloned());
			}
		}

		result.names.sort();
		result.names.dedup();
		result
	}

	/// Merge branches for conjunctive constraints.
	#[allow(clippy::unused_self)]
	fn merge_branches_conjunctive(&self, branches: &[BranchConstraint]) -> BranchConstraint {
		let mut result = BranchConstraint::any_dev();

		for b in branches {
			if b.exclude {
				if result.exclude {
					// Conjunctive: add all exclusions
					result.names.extend(b.names.iter().cloned());
				} else {
					// Only keep included names that aren't excluded
					result.names.retain(|n| !b.names.contains(n));
				}
			} else if result.exclude {
				// Switch to include mode with filtered names
				let new_names: Vec<String> = b
					.names
					.iter()
					.filter(|n| !result.names.contains(*n))
					.cloned()
					.collect();
				result.exclude = false;
				result.names = new_names;
			} else {
				// Conjunctive: only keep names in both
				result.names.retain(|n| b.names.contains(n));
			}
		}

		result.names.sort();
		result.names.dedup();
		result
	}

	/// Merge intervals for disjunctive constraints (union).
	#[allow(clippy::unused_self)]
	fn merge_intervals_disjunctive(&self, groups: &[Vec<Interval>]) -> Vec<Interval> {
		// Collect all interval borders
		let mut borders: Vec<Border> = Vec::new();

		for group in groups {
			for interval in group {
				borders.push(Border {
					version: interval.start().version().to_string(),
					operator: interval.start().operator(),
					is_start: true,
				});
				borders.push(Border {
					version: interval.end().version().to_string(),
					operator: interval.end().operator(),
					is_start: false,
				});
			}
		}

		// Sort borders
		borders.sort_by(|a, b| {
			let cmp = version_compare(&a.version, &b.version);
			if cmp == Ordering::Equal {
				a.sort_order().cmp(&b.sort_order())
			} else {
				cmp
			}
		});

		// Sweep line to find union intervals
		let mut active_intervals = 0;
		let mut intervals = Vec::new();
		let mut start: Option<SingleConstraint> = None;

		for border in borders {
			if border.is_start {
				if active_intervals == 0 {
					start = Some(SingleConstraint::new(border.operator, &border.version));
				}
				active_intervals += 1;
			} else {
				active_intervals -= 1;
				if active_intervals == 0 {
					if let Some(s) = start.take() {
						// Filter invalid intervals
						if !is_invalid_interval(&s, &border) {
							intervals.push(Interval::new(
								s,
								SingleConstraint::new(border.operator, &border.version),
							));
						}
					}
				}
			}
		}

		intervals
	}

	/// Merge intervals for conjunctive constraints (intersection).
	#[allow(clippy::unused_self)]
	fn merge_intervals_conjunctive(
		&self,
		groups: &[Vec<Interval>],
		stop_on_first: bool,
	) -> Vec<Interval> {
		// Collect all interval borders
		let mut borders: Vec<Border> = Vec::new();

		for group in groups {
			for interval in group {
				borders.push(Border {
					version: interval.start().version().to_string(),
					operator: interval.start().operator(),
					is_start: true,
				});
				borders.push(Border {
					version: interval.end().version().to_string(),
					operator: interval.end().operator(),
					is_start: false,
				});
			}
		}

		// Sort borders
		borders.sort_by(|a, b| {
			let cmp = version_compare(&a.version, &b.version);
			if cmp == Ordering::Equal {
				a.sort_order().cmp(&b.sort_order())
			} else {
				cmp
			}
		});

		// Sweep line to find intersection intervals
		let mut active_intervals = 0;
		let activation_threshold = groups.len();
		let mut intervals = Vec::new();
		let mut start: Option<SingleConstraint> = None;

		for border in borders {
			if border.is_start {
				active_intervals += 1;
			} else {
				active_intervals -= 1;
			}

			if start.is_none() && active_intervals >= activation_threshold {
				start = Some(SingleConstraint::new(border.operator, &border.version));
			} else if start.is_some() && active_intervals < activation_threshold {
				if let Some(s) = start.take() {
					// Filter invalid intervals
					if !is_invalid_interval(&s, &border) {
						intervals.push(Interval::new(
							s,
							SingleConstraint::new(border.operator, &border.version),
						));

						if stop_on_first {
							break;
						}
					}
				}
			}
		}

		intervals
	}

	/// Intersect two interval results.
	fn intersect_intervals(&self, a: &IntervalResult, b: &IntervalResult) -> IntervalResult {
		let groups = vec![a.numeric.clone(), b.numeric.clone()];
		let numeric = self.merge_intervals_conjunctive(&groups, false);
		let branches = self.merge_branches_conjunctive(&[a.branches.clone(), b.branches.clone()]);
		IntervalResult::new(numeric, branches)
	}

	/// Intersect with early exit.
	fn intersect_intervals_early_exit(
		&self,
		a: &IntervalResult,
		b: &IntervalResult,
	) -> IntervalResult {
		let groups = vec![a.numeric.clone(), b.numeric.clone()];
		let numeric = self.merge_intervals_conjunctive(&groups, true);
		let branches = self.merge_branches_conjunctive(&[a.branches.clone(), b.branches.clone()]);
		IntervalResult::new(numeric, branches)
	}
}

/// Border for interval sweep line algorithm.
#[derive(Debug)]
struct Border {
	version: String,
	operator: Operator,
	is_start: bool,
}

impl Border {
	const fn sort_order(&self) -> i32 {
		match self.operator {
			Operator::Ge => -3,
			Operator::Lt => -2,
			Operator::Gt => 2,
			Operator::Le => 3,
			Operator::Eq | Operator::Ne => 0,
		}
	}
}

/// Check if interval is invalid (e.g., > x to <= x with same x).
fn is_invalid_interval(start: &SingleConstraint, border: &Border) -> bool {
	if version_compare(start.version(), &border.version) != Ordering::Equal {
		return false;
	}

	// Invalid if: > x to <= x, or >= x to < x
	(start.operator() == Operator::Gt && border.operator == Operator::Le)
		|| (start.operator() == Operator::Ge && border.operator == Operator::Lt)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_single_constraint_intervals() {
		let mut intervals = Intervals::new();

		let c = SingleConstraint::new(Operator::Ge, "1.0.0.0-dev");
		let result = intervals.get(&c);
		assert_eq!(result.numeric.len(), 1);
		assert_eq!(result.numeric[0].start().operator(), Operator::Ge);
	}

	#[test]
	fn test_have_intersections() {
		let mut intervals = Intervals::new();

		let c1 = SingleConstraint::new(Operator::Ge, "1.0.0.0");
		let c2 = SingleConstraint::new(Operator::Lt, "2.0.0.0");
		assert!(intervals.have_intersections(&c1, &c2));

		let c3 = SingleConstraint::new(Operator::Gt, "3.0.0.0");
		let c4 = SingleConstraint::new(Operator::Lt, "2.0.0.0");
		assert!(!intervals.have_intersections(&c3, &c4));
	}
}

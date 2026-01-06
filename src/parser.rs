use crate::{
	constraint::{Constraint, MatchAllConstraint, MultiConstraint, Operator, SingleConstraint},
	error::{Result, SemverError},
	version::{expand_stability, Stability},
};
use regex::Regex;
use std::{cmp, sync::LazyLock};

/// Pre-release modifier regex pattern.
/// Matches: -beta, -b, -RC, -alpha, -a, -patch, -pl, -p with optional numbers
const MODIFIER_PATTERN: &str =
	r"[._-]?(?:(stable|beta|b|RC|alpha|a|patch|pl|p)((?:[.-]?\d+)*)?)?([.-]?dev)?";

/// Stability extraction regex (matches modifier at end of version string)
static STABILITY_EXTRACT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(&format!(r"(?i){MODIFIER_PATTERN}(?:\+.*)?$"))
		.expect("Invalid stability extract regex")
});

/// Classical versioning: v1.2.3.4-beta5
static CLASSICAL_VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(&format!(
		r"(?i)^v?(\d{{1,5}})(\.\d+)?(\.\d+)?(\.\d+)?{MODIFIER_PATTERN}$"
	))
	.expect("Invalid classical version regex")
});

/// Date-based versioning: 2010.01.02
static DATE_VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(&format!(
		r"(?i)^v?(\d{{4}}(?:[.:-]?\d{{2}}){{1,6}}(?:[.:-]?\d{{1,3}}){{0,2}}){MODIFIER_PATTERN}$"
	))
	.expect("Invalid date version regex")
});

/// Numeric branch pattern: v1.x, 2.0.*, etc.
static NUMERIC_BRANCH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?i)^v?(\d+)(\.(?:\d+|[xX*]))?(\.(?:\d+|[xX*]))?(\.(?:\d+|[xX*]))?$")
		.expect("Invalid numeric branch regex")
});

/// Alias pattern: "1.0 as 2.0"
static ALIAS_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^([^,\s]+)\s+as\s+([^,\s]+)$").expect("Invalid alias regex"));

/// Stability flag pattern: "@dev", "@stable", etc.
static STABILITY_FLAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?i)@(stable|RC|beta|alpha|dev)$").expect("Invalid stability flag regex")
});

/// Build metadata pattern: "+build123"
static BUILD_METADATA_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^([^,\s+]+)\+[^\s]+$").expect("Invalid build metadata regex"));

/// OR constraint splitter
static OR_SPLITTER: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"\s*\|\|?\s*").expect("Invalid OR splitter regex"));

/// Match all wildcard pattern
static MATCH_ALL_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?i)^(v)?[xX*](\.[xX*])*$").expect("Invalid match all regex"));

/// Version regex for constraint parsing (with capture groups for version parts)
const VERSION_REGEX_PATTERN: &str = r"v?(\d++)(?:\.(\d++))?(?:\.(\d++))?(?:\.(\d++))?(?:[._-]?(?:(stable|beta|b|RC|alpha|a|patch|pl|p)((?:[.-]?\d+)*)?)?([.-]?dev)?|\.([xX*][.-]?dev))(?:\+[^\s]+)?";

/// Tilde constraint regex
static TILDE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(&format!(r"(?i)^~>?{VERSION_REGEX_PATTERN}$")).expect("Invalid tilde regex")
});

/// Caret constraint regex
static CARET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(&format!(r"(?i)^\^{VERSION_REGEX_PATTERN}($)")).expect("Invalid caret regex")
});

/// X-range constraint regex
static XRANGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?i)^v?(\d++)(?:\.(\d++))?(?:\.(\d++))?(?:\.[xX*])++$")
		.expect("Invalid x-range regex")
});

/// Hyphen range constraint regex
static HYPHEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(&format!(
		r"(?i)^(?P<from>{VERSION_REGEX_PATTERN}) +- +(?P<to>{VERSION_REGEX_PATTERN})($)"
	))
	.expect("Invalid hyphen regex")
});

/// Basic comparator regex
static COMPARATOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"^(<>|!=|>=?|<=?|==?)?\s*(.*)").expect("Invalid comparator regex")
});

/// Stability flag in constraint
static STABILITY_CONSTRAINT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?i)^([^,\s]*?)@(stable|RC|beta|alpha|dev)$")
		.expect("Invalid stability constraint regex")
});

/// Ref stripping regex (for dev branches with #ref)
static REF_STRIP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?i)^(dev-[^,\s@]+?|[^,\s@]+?\.x-dev)#.+$").expect("Invalid ref strip regex")
});

/// Version parser for Composer semver.
#[derive(Debug, Clone, Default)]
pub struct VersionParser;

impl VersionParser {
	/// Parse and return the stability of a version string.
	pub fn parse_stability(version: &str) -> Stability {
		// Strip refs like #abcd123
		let version = version.split('#').next().unwrap_or(version);

		// dev-* prefix or *-dev suffix
		if version.to_lowercase().starts_with("dev-") || version.ends_with("-dev") {
			return Stability::Dev;
		}

		// Use regex to extract modifier from version
		let lower = version.to_lowercase();
		if let Some(caps) = STABILITY_EXTRACT_REGEX.captures(&lower) {
			// Check for dev suffix (group 3)
			if caps.get(3).is_some() {
				return Stability::Dev;
			}

			// Check for stability marker (group 1)
			if let Some(stability_match) = caps.get(1) {
				return match stability_match.as_str() {
					"rc" => Stability::RC,
					"beta" | "b" => Stability::Beta,
					"alpha" | "a" => Stability::Alpha,
					_ => Stability::Stable,
				};
			}
		}

		Stability::Stable
	}

	/// Normalize a stability string.
	///
	/// # Errors
	/// Returns `SemverError::InvalidStability` if the string is not a valid stability.
	pub fn normalize_stability(stability: &str) -> Result<String> {
		let lower = stability.to_lowercase();
		match lower.as_str() {
			"rc" => Ok("RC".to_string()),
			"dev" => Ok("dev".to_string()),
			"beta" => Ok("beta".to_string()),
			"alpha" => Ok("alpha".to_string()),
			"stable" => Ok("stable".to_string()),
			_ => Err(SemverError::InvalidStability(stability.to_string())),
		}
	}

	/// Check if a version string is valid.
	#[must_use]
	pub fn is_valid(version: &str) -> bool {
		Self::normalize(version).is_ok()
	}

	/// Normalize a version string to Composer's canonical form.
	///
	/// # Errors
	/// Returns an error if the version string is not a valid version format.
	pub fn normalize(version: &str) -> Result<String> {
		Self::normalize_full(version, None)
	}

	/// Normalize a version string with optional full version context for error messages.
	///
	/// # Errors
	/// Returns an error if the version string is not a valid version format.
	#[allow(
		clippy::missing_panics_doc,
		reason = "All unwraps are safe due to regex match checks"
	)]
	pub fn normalize_full(version: &str, full_version: Option<&str>) -> Result<String> {
		let version = version.trim();
		let orig_version = version;
		let full_version = full_version.unwrap_or(version);

		// Strip off aliasing: "1.0 as 2.0" -> "1.0"
		let version = ALIAS_REGEX
			.captures(version)
			.map_or(version, |caps| caps.get(1).map_or(version, |m| m.as_str()));

		// Strip off stability flag: "1.0@dev" -> "1.0"
		let version = STABILITY_FLAG_REGEX
			.captures(version)
			.map_or(version, |caps| {
				&version[..version.len() - caps.get(0).unwrap().as_str().len()]
			});

		// Normalize master/trunk/default branches to dev-name for BC
		let version = match version.to_lowercase().as_str() {
			"master" | "trunk" | "default" => {
				return Ok(format!("dev-{}", version.to_lowercase()));
			},
			_ => version,
		};

		// If requirement is branch-like, use full name
		if version.to_lowercase().starts_with("dev-") {
			return Ok(format!("dev-{}", &version[4..]));
		}

		// Strip off build metadata: "1.0+build123" -> "1.0"
		let version = BUILD_METADATA_REGEX
			.captures(version)
			.map_or(version, |caps| caps.get(1).map_or(version, |m| m.as_str()));

		// Try classical versioning: v1.2.3.4-beta5
		if let Some(normalized) = Self::try_classical_version(version) {
			return Ok(normalized);
		}

		// Try date-based versioning: 2010.01.02
		if let Some(normalized) = Self::try_date_version(version) {
			return Ok(normalized);
		}

		// Try dev branches: 1.x-dev, foo-dev
		if let Some(normalized) = Self::try_dev_branch(version) {
			return Ok(normalized);
		}

		Err(Self::make_error(orig_version, full_version))
	}

	/// Try to parse as classical version (1.2.3.4-beta5)
	fn try_classical_version(version: &str) -> Option<String> {
		let caps = CLASSICAL_VERSION_REGEX.captures(version)?;

		let major = caps.get(1)?.as_str();
		let minor = caps
			.get(2)
			.map_or(".0", |m| m.as_str())
			.trim_start_matches('.');
		let patch = caps
			.get(3)
			.map_or(".0", |m| m.as_str())
			.trim_start_matches('.');
		let extra = caps
			.get(4)
			.map_or(".0", |m| m.as_str())
			.trim_start_matches('.');

		let mut result = format!(
			"{}.{}.{}.{}",
			major,
			if minor.is_empty() { "0" } else { minor },
			if patch.is_empty() { "0" } else { patch },
			if extra.is_empty() { "0" } else { extra }
		);

		// Add stability modifier
		if let Some(stability_match) = caps.get(5) {
			let stability = stability_match.as_str();
			if stability.to_lowercase() != "stable" {
				let expanded = expand_stability(stability);
				result.push('-');
				result.push_str(expanded);

				// Add stability number if present
				if let Some(stability_num) = caps.get(6) {
					let num = stability_num.as_str().trim_start_matches(['.', '-']);
					if !num.is_empty() {
						result.push_str(num);
					}
				}
			}
		}

		// Add dev suffix if present
		if caps.get(7).is_some() {
			result.push_str("-dev");
		}

		Some(result)
	}

	/// Try to parse as date-based version (2010.01.02)
	fn try_date_version(version: &str) -> Option<String> {
		let caps = DATE_VERSION_REGEX.captures(version)?;

		let date_part = caps.get(1)?.as_str();
		// Replace non-digit separators with dots
		let normalized_date: String = date_part
			.chars()
			.map(|c| if c.is_ascii_digit() { c } else { '.' })
			.collect();

		// Remove consecutive dots and trailing dots
		let mut result = String::new();
		let mut last_was_dot = false;
		for c in normalized_date.chars() {
			if c == '.' {
				if !last_was_dot {
					result.push(c);
					last_was_dot = true;
				}
			} else {
				result.push(c);
				last_was_dot = false;
			}
		}

		// Trim trailing dots
		while result.ends_with('.') {
			result.pop();
		}

		// Add stability modifier if present
		if let Some(stability_match) = caps.get(2) {
			let stability = stability_match.as_str();
			if stability.to_lowercase() != "stable" {
				let expanded = expand_stability(stability);
				result.push('-');
				result.push_str(expanded);

				if let Some(stability_num) = caps.get(3) {
					let num = stability_num.as_str().trim_start_matches(['.', '-']);
					if !num.is_empty() {
						result.push_str(num);
					}
				}
			}
		}

		// Add dev suffix if present
		if caps.get(4).is_some() {
			result.push_str("-dev");
		}

		Some(result)
	}

	/// Try to parse as dev branch (1.x-dev, foo-dev)
	#[allow(clippy::case_sensitive_file_extension_comparisons)]
	fn try_dev_branch(version: &str) -> Option<String> {
		let lower = version.to_lowercase();
		if !lower.ends_with("-dev") && !lower.ends_with(".dev") && !lower.ends_with("_dev") {
			return None;
		}

		// Extract the base part (without -dev suffix) - all endings are 4 chars
		let base = &version[..version.len() - 4];

		// Try to normalize as a branch
		let normalized = Self::normalize_branch(base);

		// Only return if it's a numeric branch (not dev-prefixed)
		if normalized.starts_with("dev-") {
			None
		} else {
			Some(normalized)
		}
	}

	/// Normalize a branch name.
	pub fn normalize_branch(name: &str) -> String {
		let name = name.trim();

		// Try numeric branch pattern: v1.x, 2.0.*, etc.
		if let Some(caps) = NUMERIC_BRANCH_REGEX.captures(name) {
			let mut parts = Vec::with_capacity(4);

			for i in 1..=4 {
				let part = caps
					.get(i)
					.map_or("x", |m| m.as_str().trim_start_matches('.'));

				let lower = part.to_lowercase();
				let normalized = match lower.as_str() {
					"*" | "x" => "9999999".to_string(),
					_ => part.to_string(),
				};
				parts.push(normalized);
			}

			// Fill remaining parts with 9999999
			while parts.len() < 4 {
				parts.push("9999999".to_string());
			}

			// Replace any remaining x/* with 9999999
			let version = parts.join(".");
			let version = version.replace(['x', 'X', '*'], "9999999");

			return format!("{version}-dev");
		}

		// Non-numeric branch: feature-foo -> dev-feature-foo
		format!("dev-{name}")
	}

	/// Normalize a default branch name (master, trunk, default) to 9999999-dev.
	///
	/// This is deprecated in Composer 2 but still needed for sorting versions.
	#[deprecated(
		note = "No longer needed in Composer 2, which doesn't normalize branch names to 9999999-dev"
	)]
	#[must_use]
	pub fn normalize_default_branch(name: &str) -> String {
		if name == "dev-master" || name == "dev-default" || name == "dev-trunk" {
			"9999999-dev".to_string()
		} else {
			name.to_string()
		}
	}

	/// Extract numeric alias prefix from a branch name.
	#[must_use]
	#[allow(clippy::missing_panics_doc)]
	pub fn parse_numeric_alias_prefix(branch: &str) -> Option<String> {
		let re = Regex::new(r"(?i)^(?P<version>(\d+\.)*\d+)(?:\.x)?-dev$").expect("Invalid regex");
		if let Some(caps) = re.captures(branch) {
			if let Some(version) = caps.name("version") {
				return Some(format!("{}.", version.as_str()));
			}
		}
		None
	}

	/// Generate an error with helpful context.
	fn make_error(orig_version: &str, full_version: &str) -> SemverError {
		let mut extra_message = String::new();

		// Check if this is an alias issue
		if full_version.contains(" as ") {
			if full_version.ends_with(&format!(" as {orig_version}")) {
				extra_message =
					format!(" in \"{full_version}\", the alias must be an exact version");
			} else if full_version.starts_with(&format!("{orig_version} as ")) {
				extra_message = format!(
                    " in \"{full_version}\", the alias source must be an exact version, if it is a branch name you should prefix it with dev-"
                );
			}
		}

		SemverError::InvalidVersionWithContext {
			version: orig_version.to_string(),
			context: extra_message,
		}
	}

	/// Parse version constraints string into a Constraint.
	///
	/// # Errors
	/// Returns an error if the constraint string is empty or malformed.
	#[allow(clippy::missing_panics_doc)]
	pub fn parse_constraints(constraints: &str) -> Result<Box<dyn Constraint>> {
		let constraints = constraints.trim();
		if constraints.is_empty() {
			return Err(SemverError::EmptyConstraint);
		}

		let pretty_constraint = constraints.to_string();

		// Check for consecutive OR operators (triple pipes)
		if constraints.contains("|||") {
			return Err(SemverError::ConstraintParseFailed(
				constraints.to_string(),
				"Consecutive OR operators".to_string(),
			));
		}

		// Split on OR operators (|| or |)
		let or_parts: Vec<&str> = OR_SPLITTER.split(constraints).collect();

		// Check for leading/trailing operators
		if or_parts.first().is_some_and(|s| s.is_empty()) {
			return Err(SemverError::ConstraintParseFailed(
				constraints.to_string(),
				"Leading || or | operator".to_string(),
			));
		}
		if or_parts.last().is_some_and(|s| s.is_empty()) {
			return Err(SemverError::ConstraintParseFailed(
				constraints.to_string(),
				"Trailing || or | operator".to_string(),
			));
		}

		let mut or_groups: Vec<Box<dyn Constraint>> = Vec::new();

		for or_constraint in or_parts {
			let or_constraint = or_constraint.trim();
			if or_constraint.is_empty() {
				return Err(SemverError::ConstraintParseFailed(
					constraints.to_string(),
					"Empty constraint in OR group".to_string(),
				));
			}

			// Check for consecutive operators (double comma, triple pipe, etc.)
			if or_constraint.contains(",,")
				|| or_constraint.contains(", ,")
				|| or_constraint.contains(" ,,")
			{
				return Err(SemverError::ConstraintParseFailed(
					constraints.to_string(),
					"Consecutive comma operators".to_string(),
				));
			}

			// Check for trailing comma
			if or_constraint.trim_end().ends_with(',') {
				return Err(SemverError::ConstraintParseFailed(
					constraints.to_string(),
					"Trailing comma operator".to_string(),
				));
			}

			// Check for leading comma
			if or_constraint.trim_start().starts_with(',') {
				return Err(SemverError::ConstraintParseFailed(
					constraints.to_string(),
					"Leading comma operator".to_string(),
				));
			}

			// Split on AND operators (comma or space)
			let and_parts: Vec<&str> = Self::split_and_constraints(or_constraint);

			// Check for leading/trailing AND operators
			if and_parts.first().is_some_and(|s| s.is_empty()) {
				return Err(SemverError::ConstraintParseFailed(
					constraints.to_string(),
					"Leading comma operator".to_string(),
				));
			}
			if and_parts.last().is_some_and(|s| s.is_empty()) {
				return Err(SemverError::ConstraintParseFailed(
					constraints.to_string(),
					"Trailing comma operator".to_string(),
				));
			}

			let mut constraint_objects: Vec<Box<dyn Constraint>> = Vec::new();

			for and_constraint in and_parts {
				let and_constraint = and_constraint.trim();
				if and_constraint.is_empty() {
					return Err(SemverError::ConstraintParseFailed(
						constraints.to_string(),
						"Empty constraint in AND group (possibly double comma)".to_string(),
					));
				}
				let parsed = Self::parse_constraint(and_constraint)?;
				for c in parsed {
					constraint_objects.push(c);
				}
			}

			let constraint: Box<dyn Constraint> = if constraint_objects.len() == 1 {
				constraint_objects.pop().unwrap()
			} else {
				Box::new(MultiConstraint::new(constraint_objects, true))
			};

			or_groups.push(constraint);
		}

		let mut parsed_constraint = MultiConstraint::create(or_groups, false);
		parsed_constraint.set_pretty_string(pretty_constraint);

		Ok(parsed_constraint)
	}

	/// Split a constraint string on AND operators (comma or space).
	fn split_and_constraints(constraint: &str) -> Vec<&str> {
		// Simple split - we need to handle spaces that are not part of operators
		let mut parts = Vec::new();
		let mut current_start = 0;
		let mut in_hyphen_range = false;
		let chars: Vec<char> = constraint.chars().collect();
		let len = chars.len();
		let mut i = 0;

		while i < len {
			let c = chars[i];

			// Check for hyphen range (contains " - ")
			if c == ' ' && i + 2 < len && chars[i + 1] == '-' && chars[i + 2] == ' ' {
				in_hyphen_range = true;
				i += 3;
				continue;
			}

			// Check for " as " (alias) - don't split on this
			if c == ' ' && i + 4 < len {
				let next_chars: String = chars[i..i + 4].iter().collect();
				if next_chars == " as " {
					i += 4;
					continue;
				}
			}

			// Check for comma or space separator
			if (c == ',' || c == ' ') && !in_hyphen_range {
				// Skip if this is after an operator or 'as'
				let before = &constraint[current_start..i].trim_end();
				let after_start = i + 1;
				let after = if after_start < len {
					constraint[after_start..].trim_start()
				} else {
					""
				};

				// Don't split if next part starts with "as "
				if after.starts_with("as ") {
					i += 1;
					continue;
				}

				if !before.is_empty()
					&& !before.ends_with('>')
					&& !before.ends_with('<')
					&& !before.ends_with('=')
					&& !before.ends_with("as")
				{
					let part = constraint[current_start..i].trim();
					if !part.is_empty() {
						parts.push(part);
					}

					// Skip consecutive separators
					while i < len && (chars[i] == ',' || chars[i] == ' ') {
						i += 1;
					}
					current_start = i;
					in_hyphen_range = false;
					continue;
				}
			}

			i += 1;
		}

		// Add remaining part
		let remaining = constraint[current_start..].trim();
		if !remaining.is_empty() {
			parts.push(remaining);
		}

		if parts.is_empty() {
			parts.push(constraint.trim());
		}

		parts
	}

	/// Parse a single constraint (after splitting on OR/AND).
	fn parse_constraint(constraint: &str) -> Result<Vec<Box<dyn Constraint>>> {
		let constraint = constraint.trim();

		// Strip aliasing: "1.0 as 2.0" -> "1.0"
		let constraint = ALIAS_REGEX.captures(constraint).map_or(constraint, |caps| {
			caps.get(1).map_or(constraint, |m| m.as_str())
		});

		// Strip @stability flags and keep for later use
		let (constraint, stability_modifier) = STABILITY_CONSTRAINT_REGEX
			.captures(constraint)
			.map_or((constraint, None), |caps| {
				let base = caps.get(1).map_or("", |m| m.as_str());
				let stability = caps.get(2).map_or("", |m| m.as_str());
				let base = if base.is_empty() { "*" } else { base };
				let modifier = if stability.to_lowercase() == "stable" {
					None
				} else {
					Some(stability.to_string())
				};
				(base, modifier)
			});

		// Strip #refs
		let constraint = REF_STRIP_REGEX
			.captures(constraint)
			.map_or(constraint, |caps| {
				caps.get(1).map_or(constraint, |m| m.as_str())
			});

		// Match all wildcard: *, x.x, X.X.*, etc.
		if let Some(caps) = MATCH_ALL_REGEX.captures(constraint) {
			if caps.get(1).is_some() || caps.get(2).is_some() {
				// v* or x.x -> >= 0.0.0.0-dev
				return Ok(vec![Box::new(SingleConstraint::new(
					Operator::Ge,
					"0.0.0.0-dev",
				))]);
			}
			// Just * -> MatchAll
			return Ok(vec![Box::new(MatchAllConstraint::new())]);
		}

		// Tilde range: ~1.2.3
		if let Some(caps) = TILDE_REGEX.captures(constraint) {
			return Self::parse_tilde_constraint(constraint, &caps, stability_modifier.as_ref());
		}

		// Caret range: ^1.2.3
		if let Some(caps) = CARET_REGEX.captures(constraint) {
			return Self::parse_caret_constraint(constraint, &caps, stability_modifier.as_ref());
		}

		// X-range: 1.2.x, 2.*
		if let Some(caps) = XRANGE_REGEX.captures(constraint) {
			return Ok(Self::parse_xrange_constraint(&caps));
		}

		// Hyphen range: 1.0 - 2.0
		if let Some(caps) = HYPHEN_REGEX.captures(constraint) {
			return Self::parse_hyphen_constraint(constraint, &caps);
		}

		// Basic comparators: >=1.0, <2.0, =1.5, etc.
		if let Some(caps) = COMPARATOR_REGEX.captures(constraint) {
			return Self::parse_comparator_constraint(&caps, stability_modifier.as_ref());
		}

		Err(SemverError::ConstraintParseFailed(
			constraint.to_string(),
			"Unknown constraint format".to_string(),
		))
	}

	/// Parse tilde constraint (~1.2.3)
	fn parse_tilde_constraint(
		constraint: &str,
		caps: &regex::Captures,
		_stability_modifier: Option<&String>,
	) -> Result<Vec<Box<dyn Constraint>>> {
		// Check for invalid ~> operator
		if constraint.starts_with("~>") {
			return Err(SemverError::ConstraintParseFailed(
				constraint.to_string(),
				"Invalid operator \"~>\", you probably meant to use the \"~\" operator".to_string(),
			));
		}

		// Determine position based on how many version parts are present
		let position = if caps.get(4).is_some_and(|m| !m.as_str().is_empty()) {
			4
		} else if caps.get(3).is_some_and(|m| !m.as_str().is_empty()) {
			3
		} else if caps.get(2).is_some_and(|m| !m.as_str().is_empty()) {
			2
		} else {
			1
		};

		// When matching x-dev patterns, shift position
		let position = if caps.get(8).is_some() {
			position + 1
		} else {
			position
		};

		// Calculate stability suffix
		let stability_suffix =
			if caps.get(5).is_none() && caps.get(7).is_none() && caps.get(8).is_none() {
				"-dev"
			} else {
				""
			};

		// Normalize the low version
		let low_version = Self::normalize(&format!(
			"{}{}",
			&constraint[1..], // skip the ~
			stability_suffix
		))?;
		let lower_bound = SingleConstraint::new(Operator::Ge, low_version);

		// For upper bound, increment the position of one more significance
		let high_position = cmp::max(1, position - 1);
		let high_version = format!(
			"{}-dev",
			Self::manipulate_version_string(caps, high_position, 1)
		);
		let upper_bound = SingleConstraint::new(Operator::Lt, high_version);

		Ok(vec![Box::new(lower_bound), Box::new(upper_bound)])
	}

	/// Parse caret constraint (^1.2.3)
	fn parse_caret_constraint(
		constraint: &str,
		caps: &regex::Captures,
		_stability_modifier: Option<&String>,
	) -> Result<Vec<Box<dyn Constraint>>> {
		let major = caps.get(1).map_or("0", |m| m.as_str());
		let minor = caps.get(2).map(|m| m.as_str());
		let patch = caps.get(3).map(|m| m.as_str());

		// Determine position based on left-most non-zero digit
		let position = if major != "0" || minor.is_none() || minor == Some("") {
			1
		} else if minor != Some("0") || patch.is_none() || patch == Some("") {
			2
		} else {
			3
		};

		// Calculate stability suffix
		let stability_suffix =
			if caps.get(5).is_none() && caps.get(7).is_none() && caps.get(8).is_none() {
				"-dev"
			} else {
				""
			};

		// Normalize the low version
		let low_version = Self::normalize(&format!(
			"{}{}",
			&constraint[1..], // skip the ^
			stability_suffix
		))?;
		let lower_bound = SingleConstraint::new(Operator::Ge, low_version);

		// For upper bound
		let high_version = format!("{}-dev", Self::manipulate_version_string(caps, position, 1));
		let upper_bound = SingleConstraint::new(Operator::Lt, high_version);

		Ok(vec![Box::new(lower_bound), Box::new(upper_bound)])
	}

	/// Parse X-range constraint (1.2.x, 2.*)
	fn parse_xrange_constraint(caps: &regex::Captures) -> Vec<Box<dyn Constraint>> {
		let position = if caps.get(3).is_some_and(|m| !m.as_str().is_empty()) {
			3
		} else if caps.get(2).is_some_and(|m| !m.as_str().is_empty()) {
			2
		} else {
			1
		};

		let low_version = format!("{}-dev", Self::manipulate_version_string(caps, position, 0));
		let high_version = format!("{}-dev", Self::manipulate_version_string(caps, position, 1));

		if low_version == "0.0.0.0-dev" {
			// 0.x -> only upper bound
			return vec![Box::new(SingleConstraint::new(Operator::Lt, high_version))];
		}

		vec![
			Box::new(SingleConstraint::new(Operator::Ge, low_version)),
			Box::new(SingleConstraint::new(Operator::Lt, high_version)),
		]
	}

	/// Parse hyphen range constraint (1.0 - 2.0)
	fn parse_hyphen_constraint(
		constraint: &str,
		caps: &regex::Captures,
	) -> Result<Vec<Box<dyn Constraint>>> {
		// Check for wildcards in hyphen range (not allowed)
		if constraint.contains('*') || constraint.to_lowercase().contains('x') {
			// Only x-dev is allowed
			let from_part = caps.name("from").map_or("", |m| m.as_str());
			let to_part = caps.name("to").map_or("", |m| m.as_str());

			let from_has_invalid_wildcard = (from_part.contains('*')
				|| from_part.to_lowercase().contains('x'))
				&& !from_part.to_lowercase().ends_with("-dev")
				&& !from_part.to_lowercase().ends_with(".dev");

			let to_has_invalid_wildcard = (to_part.contains('*')
				|| to_part.to_lowercase().contains('x'))
				&& !to_part.to_lowercase().ends_with("-dev")
				&& !to_part.to_lowercase().ends_with(".dev");

			if from_has_invalid_wildcard || to_has_invalid_wildcard {
				return Err(SemverError::ConstraintParseFailed(
					constraint.to_string(),
					"Wildcards in hyphen ranges require -dev suffix".to_string(),
				));
			}
		}

		let from_str = caps.name("from").map_or("", |m| m.as_str());
		let to_str = caps.name("to").map_or("", |m| m.as_str());

		// Calculate low stability suffix
		let low_stability_suffix =
			if caps.get(6).is_none() && caps.get(8).is_none() && caps.get(9).is_none() {
				"-dev"
			} else {
				""
			};

		let low_version = format!("{}{low_stability_suffix}", Self::normalize(from_str)?);
		let lower_bound = SingleConstraint::new(Operator::Ge, low_version);

		// For upper bound, check if it's a complete version or partial
		// A version is "complete" if it has patch version OR any stability marker
		// Partial versions like "1.2" get incremented (1.2 -> < 1.3.0.0-dev)
		// Complete versions like "1.2.3" or "1.2-beta" use <= exact version
		let is_complete_version = {
			// Check if the "to" part has patch or fourth component
			let has_patch = caps.get(13).is_some_and(|m| !m.as_str().is_empty());
			let has_fourth = caps.get(14).is_some_and(|m| !m.as_str().is_empty());
			// Check for any stability in the "to" part (groups 16, 17, 18 for stability, number, dev)
			let has_stability = caps.get(16).is_some_and(|m| !m.as_str().is_empty())
				|| caps.get(17).is_some_and(|m| !m.as_str().is_empty())
				|| caps.get(18).is_some_and(|m| !m.as_str().is_empty());

			// Also check if the to_str contains stability suffix explicitly
			let to_lower = to_str.to_lowercase();
			let to_has_stability = to_lower.contains("-dev")
				|| to_lower.contains(".dev")
				|| to_lower.contains("-rc")
				|| to_lower.contains("-beta")
				|| to_lower.contains("-alpha")
				|| to_lower.contains("-b")
				|| to_lower.contains("-a");

			// Complete if: has patch OR has fourth OR has stability
			has_patch || has_fourth || has_stability || to_has_stability
		};

		let upper_bound = if is_complete_version {
			// Complete version: use <=
			let high_version = Self::normalize(to_str)?;
			SingleConstraint::new(Operator::Le, high_version)
		} else {
			// Partial version: increment and use <
			// Need to figure out which position to increment
			let minor_present = caps.get(12).is_some_and(|m| !m.as_str().is_empty());
			let position = if minor_present { 2 } else { 1 };

			// Build matches array from "to" groups (11-14 for to version)
			let to_matches = [
				"",
				caps.get(11).map_or("0", |m| m.as_str()),
				caps.get(12).map_or("", |m| m.as_str()),
				caps.get(13).map_or("", |m| m.as_str()),
				caps.get(14).map_or("", |m| m.as_str()),
			];
			let high_version = format!(
				"{}-dev",
				Self::manipulate_version_array(&to_matches, position, 1)
			);
			SingleConstraint::new(Operator::Lt, high_version)
		};

		Ok(vec![Box::new(lower_bound), Box::new(upper_bound)])
	}

	/// Parse basic comparator constraint (>=1.0, <2.0, =1.5)
	fn parse_comparator_constraint(
		caps: &regex::Captures,
		stability_modifier: Option<&String>,
	) -> Result<Vec<Box<dyn Constraint>>> {
		let op_str = caps.get(1).map_or("=", |m| m.as_str());
		let version_str = caps.get(2).map_or("", |m| m.as_str()).trim();

		if version_str.is_empty() {
			return Err(SemverError::ConstraintParseFailed(
				format!("{op_str}{version_str}"),
				"Missing version after operator".to_string(),
			));
		}

		// Check if original version has stability marker (before normalization)
		let original_lower = version_str.to_lowercase();
		let has_explicit_stability = original_lower.contains("-stable")
			|| original_lower.contains("-dev")
			|| original_lower.contains("-alpha")
			|| original_lower.contains("-beta")
			|| original_lower.contains("-rc")
			|| original_lower.contains(".dev")
			|| original_lower.contains(".alpha")
			|| original_lower.contains(".beta")
			|| original_lower.contains(".rc")
			|| version_str.starts_with("dev-");

		// Try to normalize the version
		let version = match Self::normalize(version_str) {
			Ok(v) => v,
			Err(_) => {
				// Try to recover from invalid constraint like foobar-dev
				if version_str.ends_with("-dev")
					&& version_str
						.chars()
						.all(|c| c.is_alphanumeric() || c == '-' || c == '.' || c == '/')
				{
					// Convert foo-dev to dev-foo
					Self::normalize(&format!("dev-{}", &version_str[..version_str.len() - 4]))?
				} else {
					return Err(SemverError::ConstraintParseFailed(
						version_str.to_string(),
						"Invalid version".to_string(),
					));
				}
			},
		};

		let op = match op_str {
			"<>" | "!=" => Operator::Ne,
			">=" => Operator::Ge,
			"<=" => Operator::Le,
			">" => Operator::Gt,
			"<" => Operator::Lt,
			"==" | "=" | "" => Operator::Eq,
			_ => {
				return Err(SemverError::InvalidOperator(op_str.to_string()));
			},
		};

		// Apply stability modifier or add -dev suffix for < and >=
		// But only if there was no explicit stability in the original version
		let version = if op == Operator::Eq {
			version
		} else if let Some(modifier) = stability_modifier {
			if Self::parse_stability(&version) == Stability::Stable {
				format!("{version}-{modifier}")
			} else {
				version
			}
		} else if (op == Operator::Lt || op == Operator::Ge) && !has_explicit_stability {
			// Add -dev only if no stability suffix was present in original
			format!("{version}-dev")
		} else {
			version
		};

		Ok(vec![Box::new(SingleConstraint::new(op, version))])
	}

	/// Manipulate version string from regex captures.
	fn manipulate_version_string(
		caps: &regex::Captures,
		position: usize,
		increment: i32,
	) -> String {
		let mut parts: [String; 4] = [
			caps.get(1).map_or("0", |m| m.as_str()).to_string(),
			caps.get(2).map_or("0", |m| m.as_str()).to_string(),
			caps.get(3).map_or("0", |m| m.as_str()).to_string(),
			caps.get(4).map_or("0", |m| m.as_str()).to_string(),
		];

		// Fill empty parts with "0"
		for part in &mut parts {
			if part.is_empty() {
				*part = "0".to_string();
			}
		}

		// Apply increment/padding
		for i in (0..4).rev() {
			if i + 1 > position {
				parts[i] = "0".to_string();
			} else if i + 1 == position && increment != 0 {
				let val: i64 = parts[i].parse().unwrap_or(0);
				parts[i] = (val + i64::from(increment)).to_string();
			}
		}

		format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3])
	}

	/// Manipulate version from an array of parts.
	fn manipulate_version_array(parts: &[&str; 5], position: usize, increment: i32) -> String {
		let mut result: [String; 4] = [
			parts[1].to_string(),
			if parts[2].is_empty() {
				"0".to_string()
			} else {
				parts[2].to_string()
			},
			if parts[3].is_empty() {
				"0".to_string()
			} else {
				parts[3].to_string()
			},
			if parts[4].is_empty() {
				"0".to_string()
			} else {
				parts[4].to_string()
			},
		];

		// Apply increment/padding
		for i in (0..4).rev() {
			if i + 1 > position {
				result[i] = "0".to_string();
			} else if i + 1 == position && increment != 0 {
				let val: i64 = result[i].parse().unwrap_or(0);
				result[i] = (val + i64::from(increment)).to_string();
			}
		}

		format!("{}.{}.{}.{}", result[0], result[1], result[2], result[3])
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_stability() {
		assert_eq!(VersionParser::parse_stability("1.0.0"), Stability::Stable);
		assert_eq!(VersionParser::parse_stability("dev-master"), Stability::Dev);
		assert_eq!(VersionParser::parse_stability("1.0.0-dev"), Stability::Dev);
		assert_eq!(
			VersionParser::parse_stability("1.0.0-alpha"),
			Stability::Alpha
		);
		assert_eq!(
			VersionParser::parse_stability("1.0.0-beta"),
			Stability::Beta
		);
		assert_eq!(VersionParser::parse_stability("1.0.0-RC"), Stability::RC);
	}

	#[test]
	fn test_normalize_branch() {
		assert_eq!(
			VersionParser::normalize_branch("v1.x"),
			"1.9999999.9999999.9999999-dev"
		);
		assert_eq!(
			VersionParser::normalize_branch("v1.*"),
			"1.9999999.9999999.9999999-dev"
		);
		assert_eq!(
			VersionParser::normalize_branch("v1.0"),
			"1.0.9999999.9999999-dev"
		);
		assert_eq!(
			VersionParser::normalize_branch("2.0"),
			"2.0.9999999.9999999-dev"
		);
		assert_eq!(VersionParser::normalize_branch("master"), "dev-master");
		assert_eq!(
			VersionParser::normalize_branch("feature-a"),
			"dev-feature-a"
		);
	}

	#[test]
	fn test_basic_normalization() {
		assert_eq!(VersionParser::normalize("1.0.0").unwrap(), "1.0.0.0");
		assert_eq!(VersionParser::normalize("1.2.3.4").unwrap(), "1.2.3.4");
		assert_eq!(VersionParser::normalize("v1.0.0").unwrap(), "1.0.0.0");
		assert_eq!(
			VersionParser::normalize("dev-master").unwrap(),
			"dev-master"
		);
	}
}

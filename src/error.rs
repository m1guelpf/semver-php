use thiserror::Error;

/// Errors that can occur when parsing or working with versions and constraints.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SemverError {
	#[error("Invalid version string \"{0}\"")]
	InvalidVersion(String),

	#[error("Invalid version string \"{version}\"{context}")]
	InvalidVersionWithContext { version: String, context: String },

	#[error("Invalid operator \"{0}\", expected one of: =, ==, <, <=, >, >=, !=, <>")]
	InvalidOperator(String),

	#[error("Invalid stability \"{0}\", expected one of: stable, RC, beta, alpha, dev")]
	InvalidStability(String),

	#[error("Could not parse version constraint {0}: {1}")]
	ConstraintParseFailed(String, String),

	#[error("Empty constraint string")]
	EmptyConstraint,
}

/// A specialized Result type for semver operations.
pub type Result<T> = std::result::Result<T, SemverError>;

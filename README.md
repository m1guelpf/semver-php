# semver-php

> Rust implementation of [Composer's semver](https://github.com/composer/semver) library for parsing and matching version constraints.

This library implements [Composer](https://getcomposer.org)'s versioning specification, which differs from standard semver in several important ways. You likely want to use [the `semver` crate](https://crates.io/crates/semver) instead.

## Differences from Standard Semver

Composer's semver differs from [standard semver](https://semver.org/) in several ways:

1. **Four components**: Versions are normalized to `MAJOR.MINOR.PATCH.EXTRA` (e.g., `1.2.3.0`)
2. **Branch handling**: Dev branches like `dev-master` and `1.x-dev` are first-class citizens
3. **Stability modifiers**: Versions can have stability suffixes (`-dev`, `-alpha`, `-beta`, `-RC`)
4. **Short forms**: Stability can use short forms (`-a` for alpha, `-b` for beta, etc.)
5. **Tilde behavior**: `~1.2` means `>=1.2 <2.0` (not `>=1.2.0 <1.3.0` as in npm)

## Usage

```rust
use semver_php::Semver;

// Check if a version satisfies a constraint
assert!(Semver::satisfies("1.2.3", "^1.0").unwrap());
assert!(!Semver::satisfies("2.0.0", "^1.0").unwrap());
assert!(Semver::satisfies("2.0.0", ">=1.0 <3.0").unwrap());

// Filter versions by constraint
let matching = Semver::satisfied_by(&["1.0", "1.5", "2.0", "3.0"], "^1.0").unwrap();
assert_eq!(matching, vec!["1.0", "1.5"]);

// Sort versions
let sorted = Semver::sort(&["2.0", "1.0", "1.5", "3.0"]).unwrap();
assert_eq!(sorted, vec!["1.0", "1.5", "2.0", "3.0"]);
```

## Constraint Syntax

The library supports all Composer constraint formats:

### Exact Version

```rust
use semver_php::Semver;

assert!(Semver::satisfies("1.2.3", "1.2.3").unwrap());
```

### Comparison Operators

```rust
use semver_php::Semver;
assert!(Semver::satisfies("2.0.0", ">1.0").unwrap());
assert!(Semver::satisfies("0.9.0", "<1.0").unwrap());
assert!(Semver::satisfies("1.0.0", ">=1.0").unwrap());
assert!(Semver::satisfies("1.0.0", "<=1.0").unwrap());
assert!(Semver::satisfies("2.0.0", "!=1.0").unwrap());
```

### Tilde Ranges (~)

The tilde operator allows patch-level changes:

```rust
use semver_php::Semver;

assert!(Semver::satisfies("1.5.0", "~1.2").unwrap());
assert!(Semver::satisfies("1.2.5", "~1.2.3").unwrap());
assert!(!Semver::satisfies("1.3.0", "~1.2.3").unwrap());
```

### Caret Ranges (^)

The caret operator allows changes that don't modify the left-most non-zero digit:

```rust
use semver_php::Semver;

assert!(Semver::satisfies("1.9.9", "^1.2.3").unwrap());
assert!(Semver::satisfies("0.2.9", "^0.2.3").unwrap());
assert!(!Semver::satisfies("2.0.0", "^1.2.3").unwrap());
assert!(!Semver::satisfies("0.3.0", "^0.2.3").unwrap());
```

### Wildcard Ranges

```rust
use semver_php::Semver;

assert!(Semver::satisfies("2.0.0", "*").unwrap());
assert!(Semver::satisfies("1.5.0", "1.*").unwrap());
assert!(Semver::satisfies("1.2.3", "1.2.*").unwrap());
assert!(!Semver::satisfies("1.3.0", "1.2.*").unwrap());
```

### Hyphen Ranges

```rust
use semver_php::Semver;

assert!(Semver::satisfies("1.5.0", "1.0 - 2.0").unwrap());
assert!(Semver::satisfies("2.0.5", "1.0 - 2.0").unwrap());
assert!(Semver::satisfies("2.0.0", "1.0.0 - 2.0.0").unwrap());
assert!(!Semver::satisfies("2.0.1", "1.0.0 - 2.0.0").unwrap());
```

### AND Constraints

Separate constraints with a space or comma:

```rust
use semver_php::Semver;

assert!(Semver::satisfies("1.5.0", ">=1.0 <2.0").unwrap());
assert!(Semver::satisfies("1.5.0", ">=1.0, <2.0").unwrap());
```

### OR Constraints

Separate alternatives with `||`:

```rust
use semver_php::Semver;

assert!(Semver::satisfies("1.5.0", "^1.0 || ^2.0").unwrap());
assert!(Semver::satisfies("2.5.0", "^1.0 || ^2.0").unwrap());
assert!(!Semver::satisfies("3.0.0", "^1.0 || ^2.0").unwrap());
```

## Stability Handling

Composer semver includes stability modifiers:

```rust
use semver_php::VersionParser;

assert_eq!(VersionParser::normalize("1.0.0-dev").unwrap(), "1.0.0.0-dev");
assert_eq!(VersionParser::normalize("1.0.0-RC1").unwrap(), "1.0.0.0-RC1");
assert_eq!(VersionParser::normalize("dev-master").unwrap(), "dev-master");
assert_eq!(VersionParser::normalize("1.0.0-beta2").unwrap(), "1.0.0.0-beta2");
assert_eq!(VersionParser::normalize("1.0.0-alpha1").unwrap(), "1.0.0.0-alpha1");
assert_eq!(VersionParser::normalize("1.x-dev").unwrap(), "1.9999999.9999999.9999999-dev");
```

## Version Normalization

Composer normalizes versions to a 4-component format:

```rust
use semver_php::VersionParser;

assert_eq!(VersionParser::normalize("1").unwrap(), "1.0.0.0");
assert_eq!(VersionParser::normalize("1.2").unwrap(), "1.2.0.0");
assert_eq!(VersionParser::normalize("1.2.3").unwrap(), "1.2.3.0");
assert_eq!(VersionParser::normalize("v1.2.3").unwrap(), "1.2.3.0");
```

## Comparing Versions

For direct version comparison:

```rust
assert!(semver_php::compare("1.0", "<", "2.0").unwrap());
assert!(semver_php::compare("2.0", ">", "1.0").unwrap());
assert!(semver_php::compare("1.0", "==", "1.0.0").unwrap());

assert!(semver_php::less_than("1.0", "2.0").unwrap());
assert!(semver_php::greater_than("2.0", "1.0").unwrap());
assert!(semver_php::equal_to("1.0.0", "1.0.0.0").unwrap());
```

## Working with Constraints Directly

For more control, use the constraint types directly:

```rust
use semver_php::{VersionParser, Constraint, SingleConstraint, Operator};

let constraint = VersionParser::parse_constraints("^1.0 || ^2.0").unwrap();

assert!(constraint.matches(&SingleConstraint::new(Operator::Eq, "1.5.0.0")));
```

## Benchmarks

This library is benchmarked against the original PHP `composer/semver` using [Criterion](https://github.com/bheisler/criterion.rs) (Rust) and [PHPBench](https://github.com/phpbench/phpbench) (PHP).

| Operation                   | PHP     | Rust    | Speedup |
| --------------------------- | ------- | ------- | ------- |
| Normalize (simple)          | 7.7 µs  | 3.6 µs  | 2.1x    |
| Normalize (complex)         | 13.8 µs | 6.1 µs  | 2.3x    |
| Parse constraints (simple)  | 19.0 µs | 9.2 µs  | 2.1x    |
| Parse constraints (complex) | 53.2 µs | 22.4 µs | 2.4x    |
| Satisfies check             | 64.8 µs | 38.2 µs | 1.7x    |
| Sort (10 versions)          | 29.3 µs | 12.0 µs | 2.4x    |
| Sort (100 versions)         | 392 µs  | 252 µs  | 1.6x    |
| Parse stability             | 6.2 µs  | 0.9 µs  | 7.3x    |

**Geometric mean speedup: 2.1x**

To run the benchmarks yourself (requires Rust and PHP):

```bash
./benches/compare.sh
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

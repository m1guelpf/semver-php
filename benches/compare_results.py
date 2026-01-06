#!/usr/bin/env python3
"""
Compare benchmark results between Rust and PHP implementations.

Usage:
    1. Run PHP benchmark: cd benches && vendor/bin/phpbench run --dump-file=php_results.xml
    2. Run Rust benchmark: cargo bench
    3. Run this script: python3 benches/compare_results.py

This script reads PHPBench XML results and parses Criterion's output to produce
a side-by-side comparison.
"""

import json
import os
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

# Mapping from PHPBench method names to Criterion benchmark names
BENCHMARK_MAPPING = {
    "benchNormalizeSimple": "normalize/simple",
    "benchNormalizeComplex": "normalize/complex",
    "benchNormalizeSingle100": "normalize/single/1.0.0",
    "benchNormalizeSingleAlpha1": "normalize/single/1.0.0-alpha1",
    "benchNormalizeSingleDevMaster": "normalize/single/dev-master",
    "benchNormalizeSingle1xDev": "normalize/single/1.x-dev",
    "benchParseConstraintsSimple": "parse_constraints/simple",
    "benchParseConstraintsComplex": "parse_constraints/complex",
    "benchParseConstraintsSingleCaret": "parse_constraints/single/^1.0",
    "benchParseConstraintsSingleRange": "parse_constraints/single/>=1.0 <2.0",
    "benchParseConstraintsSingleMultiOr": "parse_constraints/single/^1.0 || ^2.0 || ^3.0",
    "benchSatisfiesAllCases": "satisfies/all_cases",
    "benchSatisfiesSingleCaret": "satisfies/single/1.2.3 vs ^1.0",
    "benchSatisfiesSingleOr": "satisfies/single/2.0.0 vs ^1.0 || ^2.0",
    "benchSatisfiesSingleRange": "satisfies/single/1.5.0 vs >=1.0 <2.0",
    "benchSort10Versions": "sort/10_versions",
    "benchSort100Versions": "sort/100_versions",
    "benchSatisfiedByCaret": "satisfied_by/filter/^1.0",
    "benchSatisfiedByRange": "satisfied_by/filter/>=1.0 <2.0",
    "benchSatisfiedByOr": "satisfied_by/filter/^1.0 || ^2.0",
    "benchIsValidMixed": "is_valid/mixed",
    "benchParseStabilityAll": "parse_stability/all",
}


def load_phpbench_results(path: str) -> dict:
    """Load PHPBench benchmark results from XML file."""
    results = {}
    try:
        tree = ET.parse(path)
        root = tree.getroot()

        for benchmark in root.findall(".//benchmark"):
            for subject in benchmark.findall("subject"):
                method_name = subject.get("name")
                variant = subject.find("variant")
                if variant is not None:
                    stats = variant.find("stats")
                    if stats is not None:
                        # PHPBench reports mean in microseconds
                        mean_us = float(stats.get("mean", 0))
                        mean_ns = mean_us * 1000  # Convert to nanoseconds
                        stdev_us = float(stats.get("stdev", 0))

                        # Map to Criterion naming convention
                        criterion_name = BENCHMARK_MAPPING.get(method_name, method_name)
                        results[criterion_name] = {
                            "mean_ns": mean_ns,
                            "stdev_ns": stdev_us * 1000,
                        }
    except FileNotFoundError:
        print(f"Warning: PHPBench results not found at {path}")
        print("Run: cd benches && vendor/bin/phpbench run --dump-file=php_results.xml")
    except ET.ParseError as e:
        print(f"Error parsing PHPBench XML: {e}")

    return results


def load_criterion_results(criterion_dir: str) -> dict:
    """Load Rust Criterion benchmark results."""
    results = {}
    criterion_path = Path(criterion_dir)

    if not criterion_path.exists():
        print(f"Warning: Criterion results not found at {criterion_dir}")
        print("Run: cargo bench")
        return {}

    def unescape_criterion_name(name: str) -> str:
        """Convert Criterion's escaped directory names back to original."""
        # Criterion escapes special chars: ^ -> _, || -> __, >= -> _=, < removed
        # But underscores in normal names (like all_cases) should stay
        result = name
        result = result.replace(" __ ", " || ")
        result = result.replace("_=", ">=")
        # Only replace _ with ^ when it looks like a version constraint pattern
        # e.g., "_1.0" should become "^1.0" but "all_cases" should stay
        import re
        result = re.sub(r'_(\d)', r'^\1', result)  # _1.0 -> ^1.0
        # Handle cases like ">=1.0 ^2.0" which should be ">=1.0 <2.0"
        result = re.sub(r'>=(\d+\.\d+)\s+\^(\d)', r'>=\1 <\2', result)
        return result

    def find_benchmarks(path: Path, prefix: str = "") -> dict:
        """Recursively find all benchmark results."""
        found = {}
        for item in path.iterdir():
            if not item.is_dir() or item.name == "report":
                continue

            estimates_path = item / "new" / "estimates.json"
            if estimates_path.exists():
                # This is a benchmark result
                bench_name = unescape_criterion_name(item.name)
                full_name = f"{prefix}/{bench_name}" if prefix else bench_name
                with open(estimates_path) as f:
                    data = json.load(f)
                    mean_ns = data.get("mean", {}).get("point_estimate", 0)
                    std_error = data.get("mean", {}).get("standard_error", 0)
                    found[full_name] = {
                        "mean_ns": mean_ns,
                        "stdev_ns": std_error,
                    }
            else:
                # Check for nested benchmarks
                nested_prefix = f"{prefix}/{item.name}" if prefix else item.name
                found.update(find_benchmarks(item, nested_prefix))
        return found

    results = find_benchmarks(criterion_path)
    return results


def format_time(ns: float) -> str:
    """Format nanoseconds to human-readable string."""
    if ns >= 1_000_000_000:
        return f"{ns / 1_000_000_000:.3f} s"
    elif ns >= 1_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    elif ns >= 1_000:
        return f"{ns / 1_000:.3f} µs"
    else:
        return f"{ns:.0f} ns"


def calculate_speedup(php_ns: float, rust_ns: float) -> tuple:
    """Calculate speedup ratio and return (ratio, formatted_string)."""
    if rust_ns == 0 or php_ns == 0:
        return (0, "N/A")
    ratio = php_ns / rust_ns
    if ratio >= 1:
        return (ratio, f"{ratio:.1f}x faster")
    else:
        return (1 / ratio, f"{1/ratio:.1f}x slower")


def main():
    script_dir = Path(__file__).parent
    project_dir = script_dir.parent

    phpbench_results_path = script_dir / "php_results.xml"
    criterion_dir = project_dir / "target" / "criterion"

    php_results = load_phpbench_results(str(phpbench_results_path))
    rust_results = load_criterion_results(str(criterion_dir))

    if not php_results and not rust_results:
        print("No benchmark results found. Run the benchmarks first:")
        print("  cd benches && vendor/bin/phpbench run --dump-file=php_results.xml")
        print("  cargo bench")
        sys.exit(1)

    print("=" * 85)
    print("  Benchmark Comparison: Rust semver-php vs PHP composer/semver")
    print("  (Using Criterion for Rust, PHPBench for PHP)")
    print("=" * 85)
    print()

    # Header
    print(f"{'Benchmark':<45} {'PHP':>12} {'Rust':>12} {'Speedup':>15}")
    print("-" * 85)

    # Group benchmarks by category
    categories = {}
    for criterion_name in BENCHMARK_MAPPING.values():
        category = criterion_name.split("/")[0]
        if category not in categories:
            categories[category] = []
        categories[category].append(criterion_name)

    total_php = 0
    total_rust = 0
    count = 0
    speedups = []

    for category in ["normalize", "parse_constraints", "satisfies", "sort", "satisfied_by", "is_valid", "parse_stability"]:
        if category not in categories:
            continue

        for bench_name in categories[category]:
            php_data = php_results.get(bench_name)
            rust_data = rust_results.get(bench_name)

            php_time = php_data.get("mean_ns", 0) if php_data else 0
            rust_time = rust_data.get("mean_ns", 0) if rust_data else 0

            php_str = format_time(php_time) if php_time else "N/A"
            rust_str = format_time(rust_time) if rust_time else "N/A"

            if php_time > 0 and rust_time > 0:
                ratio, speedup_str = calculate_speedup(php_time, rust_time)
                speedups.append(ratio)
                total_php += php_time
                total_rust += rust_time
                count += 1
            else:
                speedup_str = "N/A"

            print(f"{bench_name:<45} {php_str:>12} {rust_str:>12} {speedup_str:>15}")

        print()  # Empty line between categories

    # Summary
    print("=" * 85)
    if count > 0 and total_rust > 0:
        avg_speedup = total_php / total_rust
        # Geometric mean is more appropriate for speedup ratios
        from functools import reduce
        import math
        geo_mean = math.exp(sum(math.log(s) for s in speedups) / len(speedups))
        print(f"Benchmarks compared: {count}")
        print(f"Aggregate speedup (total time): {avg_speedup:.1f}x")
        print(f"Geometric mean speedup: {geo_mean:.1f}x")
    print("=" * 85)


if __name__ == "__main__":
    main()

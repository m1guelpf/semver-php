#!/bin/bash
#
# Benchmark comparison script for semver-php (Rust) vs composer/semver (PHP)
#
# Usage: ./benches/compare.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=============================================="
echo "  Semver Benchmark: Rust vs PHP"
echo "  (Using Criterion for Rust, PHPBench for PHP)"
echo "=============================================="
echo ""

# Check if PHP composer dependencies are installed
if [ ! -f "$SCRIPT_DIR/vendor/autoload.php" ]; then
    echo "Installing PHP benchmark dependencies..."
    composer install --working-dir="$SCRIPT_DIR" --quiet
fi

# Run PHP benchmark with PHPBench
echo "Running PHP benchmark (PHPBench)..."
echo "----------------------------------------------"
"$SCRIPT_DIR/vendor/bin/phpbench" run \
    --working-dir="$SCRIPT_DIR" \
    --report=default \
    --dump-file="$SCRIPT_DIR/php_results.xml"
echo ""

# Run Rust benchmark
echo "----------------------------------------------"
echo "Running Rust benchmark (Criterion)..."
echo "----------------------------------------------"
cargo bench --manifest-path="$PROJECT_DIR/Cargo.toml" --quiet 2>/dev/null || \
    cargo bench --manifest-path="$PROJECT_DIR/Cargo.toml"

echo ""
echo "=============================================="
echo "  Running Comparison"
echo "=============================================="
echo ""

# Run comparison script
python3 "$SCRIPT_DIR/compare_results.py"

echo ""
echo "PHP results saved to: benches/php_results.xml"
echo "Rust results saved to: target/criterion/"
echo ""
echo "To view detailed Rust benchmarks, open:"
echo "  target/criterion/report/index.html"
echo ""

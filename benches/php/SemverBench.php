<?php

declare(strict_types=1);

namespace SemverBench;

use Composer\Semver\Semver;
use Composer\Semver\VersionParser;
use PhpBench\Attributes as Bench;

/**
 * Benchmarks for Composer Semver
 *
 * Run with: cd benches && vendor/bin/phpbench run --report=default
 */
class SemverBench
{
    private VersionParser $parser;

    private const VERSIONS_SIMPLE = [
        "1.0.0", "2.0.0", "1.2.3", "0.1.0", "10.20.30", "1.0.0-alpha", "1.0.0-beta", "1.0.0-RC1",
    ];

    private const VERSIONS_COMPLEX = [
        "1.0.0",
        "v2.0.0",
        "1.2.3.4",
        "1.0.0-alpha1",
        "1.0.0-beta2",
        "1.0.0-RC1",
        "dev-master",
        "1.x-dev",
        "2.0.x-dev",
        "1.0.0+build.123",
        "2010.01.02",
        "1.0.0-dev",
    ];

    private const CONSTRAINTS_SIMPLE = [
        "^1.0",
        "~1.2",
        ">=1.0",
        "<2.0",
        "1.0.*",
        "1.*",
        "*",
        "1.0.0",
    ];

    private const CONSTRAINTS_COMPLEX = [
        "^1.0 || ^2.0",
        ">=1.0 <2.0",
        "~1.2.3",
        "^0.2.3",
        "1.0 - 2.0",
        ">=1.0, <2.0, !=1.5.0",
        "^1.0 || ^2.0 || ^3.0",
        ">=1.0 <1.1 || >=1.2 <1.3",
    ];

    private const SATISFIES_CASES = [
        ["1.2.3", "^1.0"],
        ["2.0.0", "^1.0 || ^2.0"],
        ["1.5.0", ">=1.0 <2.0"],
        ["1.2.5", "~1.2.3"],
        ["0.2.9", "^0.2.3"],
        ["1.5.0", "1.0 - 2.0"],
        ["1.2.3", "1.2.*"],
        ["2.0.0", "*"],
        ["1.0.0-alpha", ">=1.0.0-alpha <1.0.0"],
        ["1.0.0-RC1", "^1.0@RC"],
    ];

    private const SORT_VERSIONS = [
        "2.0.0",
        "1.0.0",
        "1.5.0",
        "0.1.0",
        "1.0.0-alpha",
        "1.0.0-beta",
        "1.0.0-RC1",
        "3.0.0",
        "1.2.3",
        "10.0.0",
    ];

    private const FILTER_VERSIONS = [
        "0.5.0", "1.0.0", "1.0.5", "1.1.0", "1.2.0", "1.5.0", "2.0.0", "2.5.0", "3.0.0",
    ];

    private const STABILITY_VERSIONS = [
        "1.0.0",
        "1.0.0-dev",
        "1.0.0-alpha1",
        "1.0.0-beta2",
        "1.0.0-RC1",
        "dev-master",
        "1.x-dev",
    ];

    private array $versions100;
    private array $mixedVersions;

    public function __construct()
    {
        $this->parser = new VersionParser();
        $this->versions100 = [];
        for ($i = 0; $i < 10; $i++) {
            $this->versions100 = array_merge($this->versions100, self::SORT_VERSIONS);
        }
        $this->mixedVersions = array_merge(
            self::VERSIONS_COMPLEX,
            ["not-a-version", "abc", "1.2.3.4.5.6", ">>>1.0"]
        );
    }

    // ============ Normalize benchmarks ============

    #[Bench\Groups(['normalize'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchNormalizeSimple(): void
    {
        foreach (self::VERSIONS_SIMPLE as $version) {
            $this->parser->normalize($version);
        }
    }

    #[Bench\Groups(['normalize'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchNormalizeComplex(): void
    {
        foreach (self::VERSIONS_COMPLEX as $version) {
            $this->parser->normalize($version);
        }
    }

    #[Bench\Groups(['normalize'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchNormalizeSingle100(): void
    {
        $this->parser->normalize("1.0.0");
    }

    #[Bench\Groups(['normalize'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchNormalizeSingleAlpha1(): void
    {
        $this->parser->normalize("1.0.0-alpha1");
    }

    #[Bench\Groups(['normalize'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchNormalizeSingleDevMaster(): void
    {
        $this->parser->normalize("dev-master");
    }

    #[Bench\Groups(['normalize'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchNormalizeSingle1xDev(): void
    {
        $this->parser->normalize("1.x-dev");
    }

    // ============ Parse constraints benchmarks ============

    #[Bench\Groups(['parse_constraints'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchParseConstraintsSimple(): void
    {
        foreach (self::CONSTRAINTS_SIMPLE as $constraint) {
            $this->parser->parseConstraints($constraint);
        }
    }

    #[Bench\Groups(['parse_constraints'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchParseConstraintsComplex(): void
    {
        foreach (self::CONSTRAINTS_COMPLEX as $constraint) {
            $this->parser->parseConstraints($constraint);
        }
    }

    #[Bench\Groups(['parse_constraints'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchParseConstraintsSingleCaret(): void
    {
        $this->parser->parseConstraints("^1.0");
    }

    #[Bench\Groups(['parse_constraints'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchParseConstraintsSingleRange(): void
    {
        $this->parser->parseConstraints(">=1.0 <2.0");
    }

    #[Bench\Groups(['parse_constraints'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchParseConstraintsSingleMultiOr(): void
    {
        $this->parser->parseConstraints("^1.0 || ^2.0 || ^3.0");
    }

    // ============ Satisfies benchmarks ============

    #[Bench\Groups(['satisfies'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiesAllCases(): void
    {
        foreach (self::SATISFIES_CASES as [$version, $constraint]) {
            Semver::satisfies($version, $constraint);
        }
    }

    #[Bench\Groups(['satisfies'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiesSingleCaret(): void
    {
        Semver::satisfies("1.2.3", "^1.0");
    }

    #[Bench\Groups(['satisfies'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiesSingleOr(): void
    {
        Semver::satisfies("2.0.0", "^1.0 || ^2.0");
    }

    #[Bench\Groups(['satisfies'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiesSingleRange(): void
    {
        Semver::satisfies("1.5.0", ">=1.0 <2.0");
    }

    // ============ Sort benchmarks ============

    #[Bench\Groups(['sort'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSort10Versions(): void
    {
        Semver::sort(self::SORT_VERSIONS);
    }

    #[Bench\Groups(['sort'])]
    #[Bench\Revs(100)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSort100Versions(): void
    {
        Semver::sort($this->versions100);
    }

    // ============ Satisfied by benchmarks ============

    #[Bench\Groups(['satisfied_by'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiedByCaret(): void
    {
        Semver::satisfiedBy(self::FILTER_VERSIONS, "^1.0");
    }

    #[Bench\Groups(['satisfied_by'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiedByRange(): void
    {
        Semver::satisfiedBy(self::FILTER_VERSIONS, ">=1.0 <2.0");
    }

    #[Bench\Groups(['satisfied_by'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchSatisfiedByOr(): void
    {
        Semver::satisfiedBy(self::FILTER_VERSIONS, "^1.0 || ^2.0");
    }

    // ============ Is valid benchmarks ============

    #[Bench\Groups(['is_valid'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchIsValidMixed(): void
    {
        foreach ($this->mixedVersions as $version) {
            try {
                $this->parser->normalize($version);
            } catch (\Exception $e) {
                // Invalid version
            }
        }
    }

    // ============ Parse stability benchmarks ============

    #[Bench\Groups(['parse_stability'])]
    #[Bench\Revs(1000)]
    #[Bench\Iterations(10)]
    #[Bench\Warmup(3)]
    public function benchParseStabilityAll(): void
    {
        foreach (self::STABILITY_VERSIONS as $version) {
            VersionParser::parseStability($version);
        }
    }
}

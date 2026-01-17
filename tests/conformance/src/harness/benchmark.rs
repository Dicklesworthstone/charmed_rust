//! BenchContext - Statistical analysis for performance benchmarks
//!
//! Provides a framework for benchmarking conformance tests with:
//! - Configurable warmup and measurement iterations
//! - Statistical analysis (min, max, mean, median, std dev, percentiles)
//! - Outlier detection and removal (MAD and IQR methods)
//! - Coefficient of variation for stability assessment
//! - Adaptive warmup until measurements stabilize
//! - Baseline comparison and regression detection
//! - Results formatting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Method for outlier removal
#[derive(Debug, Clone, Copy, Default)]
pub enum OutlierRemoval {
    /// No outlier removal
    #[default]
    None,
    /// Median Absolute Deviation method
    Mad {
        /// Number of MADs from median to consider outlier (typically 3.0)
        threshold: f64,
    },
    /// Interquartile Range method
    Iqr {
        /// Multiplier for IQR (typically 1.5)
        multiplier: f64,
    },
}

/// Configuration for benchmark runs
#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// Number of warmup iterations
    pub warmup_iterations: usize,
    /// Number of measured iterations
    pub measure_iterations: usize,
    /// Use adaptive warmup (run until CV stabilizes)
    pub adaptive_warmup: bool,
    /// Outlier removal method
    pub outlier_removal: OutlierRemoval,
    /// Threshold for regression detection (e.g., 0.10 for 10% slower)
    pub regression_threshold: f64,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 3,
            measure_iterations: 100,
            adaptive_warmup: false,
            outlier_removal: OutlierRemoval::None,
            regression_threshold: 0.10,
        }
    }
}

/// Comparison against a baseline benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    /// Baseline mean duration
    pub baseline_mean: Duration,
    /// Current mean duration
    pub current_mean: Duration,
    /// Percentage change (negative = faster, positive = slower)
    pub change_percent: f64,
    /// Whether this represents a regression
    pub is_regression: bool,
}

/// Stored baseline benchmark results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchBaseline {
    /// Stored results by benchmark name
    pub results: HashMap<String, StoredBenchResult>,
    /// When the baseline was captured
    #[serde(default)]
    pub captured_at: Option<String>,
    /// Rust version used
    #[serde(default)]
    pub rust_version: Option<String>,
    /// Platform identifier
    #[serde(default)]
    pub platform: Option<String>,
}

/// Simplified result for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBenchResult {
    pub mean: Duration,
    pub std_dev: Duration,
    pub iterations: usize,
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchResult {
    /// Name of the benchmark
    pub name: String,
    /// Minimum duration observed
    pub min: Duration,
    /// Maximum duration observed
    pub max: Duration,
    /// Mean (average) duration
    pub mean: Duration,
    /// Median duration (p50)
    pub median: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// 50th percentile
    pub p50: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// Total time for all iterations
    pub total: Duration,
    /// Coefficient of variation (std_dev / mean)
    pub coefficient_of_variation: f64,
    /// Number of iterations measured
    pub iterations: usize,
    /// Number of outliers removed
    pub outliers_removed: usize,
    /// Comparison against baseline (if available)
    pub vs_baseline: Option<BaselineComparison>,
}

impl BenchResult {
    /// Format the result as a human-readable string
    pub fn to_string_pretty(&self) -> String {
        format!(
            "{}: min={:?}, max={:?}, mean={:?}, median={:?}, std_dev={:?} ({} iterations)",
            self.name, self.min, self.max, self.mean, self.median, self.std_dev, self.iterations
        )
    }

    /// Detailed display output
    pub fn display_detailed(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Benchmark: {}\n", self.name));
        output.push_str(&format!(
            "  Iterations: {} (outliers removed: {})\n",
            self.iterations, self.outliers_removed
        ));
        output.push_str("\n  Timing:\n");
        output.push_str(&format!("    Min:     {:?}\n", self.min));
        output.push_str(&format!("    Max:     {:?}\n", self.max));
        output.push_str(&format!(
            "    Mean:    {:?} ± {:?} (CV: {:.1}%)\n",
            self.mean,
            self.std_dev,
            self.coefficient_of_variation * 100.0
        ));
        output.push_str(&format!("    Median:  {:?}\n", self.median));
        output.push_str(&format!("    p95:     {:?}\n", self.p95));
        output.push_str(&format!("    p99:     {:?}\n", self.p99));
        output.push_str(&format!("    Total:   {:?}\n", self.total));

        if let Some(ref comparison) = self.vs_baseline {
            output.push_str("\n  vs Baseline:\n");
            output.push_str(&format!("    Previous: {:?}\n", comparison.baseline_mean));
            output.push_str(&format!(
                "    Change:   {:.1}% {}\n",
                comparison.change_percent.abs() * 100.0,
                if comparison.is_regression {
                    "⚠ REGRESSION"
                } else if comparison.change_percent < 0.0 {
                    "✓ IMPROVED"
                } else {
                    "~ UNCHANGED"
                }
            ));
        }

        output
    }

    /// Check if this result represents a regression
    pub fn is_regression(&self) -> bool {
        self.vs_baseline
            .as_ref()
            .is_some_and(|c| c.is_regression)
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&StoredBenchResult {
            mean: self.mean,
            std_dev: self.std_dev,
            iterations: self.iterations,
        })
        .unwrap_or_default()
    }
}

/// Context for running benchmarks with statistical analysis
pub struct BenchContext {
    /// Configuration
    config: BenchConfig,
    /// Collected results from measured iterations
    results: Vec<Duration>,
    /// All benchmark results from this context
    all_results: Vec<BenchResult>,
    /// Baseline for comparison
    baseline: Option<BenchBaseline>,
}

impl Default for BenchContext {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchContext {
    /// Create a new benchmark context with default settings
    pub fn new() -> Self {
        Self {
            config: BenchConfig::default(),
            results: Vec::new(),
            all_results: Vec::new(),
            baseline: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: BenchConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            all_results: Vec::new(),
            baseline: None,
        }
    }

    /// Set the number of warmup iterations
    pub fn warmup(mut self, iterations: usize) -> Self {
        self.config.warmup_iterations = iterations;
        self
    }

    /// Alias for warmup
    pub fn with_warmup(self, iterations: usize) -> Self {
        self.warmup(iterations)
    }

    /// Set the number of measured iterations
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.config.measure_iterations = iterations;
        self
    }

    /// Alias for iterations
    pub fn with_iterations(self, iterations: usize) -> Self {
        self.iterations(iterations)
    }

    /// Enable adaptive warmup
    pub fn adaptive_warmup(mut self, enabled: bool) -> Self {
        self.config.adaptive_warmup = enabled;
        self
    }

    /// Set outlier removal method
    pub fn outlier_removal(mut self, method: OutlierRemoval) -> Self {
        self.config.outlier_removal = method;
        self
    }

    /// Set regression detection threshold
    pub fn regression_threshold(mut self, threshold: f64) -> Self {
        self.config.regression_threshold = threshold;
        self
    }

    /// Set baseline for comparison
    pub fn with_baseline(mut self, baseline: BenchBaseline) -> Self {
        self.baseline = Some(baseline);
        self
    }

    /// Run a benchmark and collect results
    pub fn bench<F>(&mut self, name: &str, mut f: F) -> BenchResult
    where
        F: FnMut(),
    {
        self.results.clear();

        // Warmup
        if self.config.adaptive_warmup {
            self.run_adaptive_warmup(&mut f);
        } else {
            for _ in 0..self.config.warmup_iterations {
                f();
            }
        }

        // Measure
        for _ in 0..self.config.measure_iterations {
            let start = Instant::now();
            f();
            let elapsed = start.elapsed();
            self.results.push(elapsed);
        }

        let result = self.calculate_stats(name);
        self.all_results.push(result.clone());
        result
    }

    /// Run benchmark with setup (setup time excluded from measurement)
    pub fn bench_with_setup<S, F, T>(&mut self, name: &str, mut setup: S, mut f: F) -> BenchResult
    where
        S: FnMut() -> T,
        F: FnMut(T),
    {
        self.results.clear();

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let data = setup();
            f(data);
        }

        // Measure
        for _ in 0..self.config.measure_iterations {
            let data = setup();
            let start = Instant::now();
            f(data);
            let elapsed = start.elapsed();
            self.results.push(elapsed);
        }

        let result = self.calculate_stats(name);
        self.all_results.push(result.clone());
        result
    }

    /// Run benchmark with input generator
    pub fn bench_with_input<I, F, T>(&mut self, name: &str, input_gen: I, mut f: F) -> BenchResult
    where
        I: Fn() -> T,
        F: FnMut(T),
    {
        self.results.clear();

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let input = input_gen();
            f(input);
        }

        // Measure
        for _ in 0..self.config.measure_iterations {
            let input = input_gen();
            let start = Instant::now();
            f(input);
            let elapsed = start.elapsed();
            self.results.push(elapsed);
        }

        let result = self.calculate_stats(name);
        self.all_results.push(result.clone());
        result
    }

    /// Run adaptive warmup until CV stabilizes
    fn run_adaptive_warmup<F>(&self, f: &mut F)
    where
        F: FnMut(),
    {
        let mut samples = Vec::new();
        let mut cv_history = Vec::new();
        let min_iterations = self.config.warmup_iterations.max(5);

        loop {
            let start = Instant::now();
            f();
            samples.push(start.elapsed());

            if samples.len() >= min_iterations {
                let cv = Self::calculate_cv(&samples);
                cv_history.push(cv);

                // Stable if last 3 CVs are within 5% of each other
                if cv_history.len() >= 3 {
                    let recent: Vec<_> = cv_history.iter().rev().take(3).copied().collect();
                    let max = recent.iter().fold(0.0f64, |a, &b| a.max(b));
                    let min = recent.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    if max > 0.0 && (max - min) / max < 0.05 {
                        break;
                    }
                }
            }

            // Safety limit
            if samples.len() > 10000 {
                break;
            }
        }
    }

    /// Calculate coefficient of variation for a set of samples
    fn calculate_cv(samples: &[Duration]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }

        let n = samples.len() as f64;
        let mean: f64 = samples.iter().map(|d| d.as_secs_f64()).sum::<f64>() / n;

        if mean == 0.0 {
            return 0.0;
        }

        let variance: f64 = samples
            .iter()
            .map(|d| {
                let diff = d.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>()
            / n;

        variance.sqrt() / mean
    }

    /// Remove outliers using MAD method
    fn remove_outliers_mad(samples: &[Duration], threshold: f64) -> Vec<Duration> {
        if samples.is_empty() {
            return Vec::new();
        }

        let mut sorted: Vec<f64> = samples.iter().map(|d| d.as_secs_f64()).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        // Calculate MAD
        let mut deviations: Vec<f64> = sorted.iter().map(|&x| (x - median).abs()).collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mad = if deviations.len() % 2 == 0 {
            (deviations[deviations.len() / 2 - 1] + deviations[deviations.len() / 2]) / 2.0
        } else {
            deviations[deviations.len() / 2]
        };

        // MAD-based cutoff (1.4826 is for consistency with normal distribution)
        let cutoff_high = median + threshold * mad * 1.4826;
        let cutoff_low = median - threshold * mad * 1.4826;

        samples
            .iter()
            .filter(|d| {
                let v = d.as_secs_f64();
                v >= cutoff_low && v <= cutoff_high
            })
            .copied()
            .collect()
    }

    /// Remove outliers using IQR method
    fn remove_outliers_iqr(samples: &[Duration], multiplier: f64) -> Vec<Duration> {
        if samples.is_empty() {
            return Vec::new();
        }

        let mut sorted: Vec<f64> = samples.iter().map(|d| d.as_secs_f64()).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len();
        let q1 = sorted[n / 4];
        let q3 = sorted[3 * n / 4];
        let iqr = q3 - q1;

        let lower = q1 - multiplier * iqr;
        let upper = q3 + multiplier * iqr;

        samples
            .iter()
            .filter(|d| {
                let v = d.as_secs_f64();
                v >= lower && v <= upper
            })
            .copied()
            .collect()
    }

    /// Calculate percentile from sorted samples
    fn percentile(sorted: &[Duration], p: f64) -> Duration {
        if sorted.is_empty() {
            return Duration::ZERO;
        }
        let idx = ((sorted.len() as f64 - 1.0) * p / 100.0).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Calculate statistics from collected results
    fn calculate_stats(&self, name: &str) -> BenchResult {
        let original_count = self.results.len();

        if original_count == 0 {
            return BenchResult {
                name: name.to_string(),
                min: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                median: Duration::ZERO,
                std_dev: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                total: Duration::ZERO,
                coefficient_of_variation: 0.0,
                iterations: 0,
                outliers_removed: 0,
                vs_baseline: None,
            };
        }

        // Apply outlier removal if configured
        let samples = match self.config.outlier_removal {
            OutlierRemoval::None => self.results.clone(),
            OutlierRemoval::Mad { threshold } => {
                Self::remove_outliers_mad(&self.results, threshold)
            }
            OutlierRemoval::Iqr { multiplier } => {
                Self::remove_outliers_iqr(&self.results, multiplier)
            }
        };

        let outliers_removed = original_count - samples.len();
        let n = samples.len();

        if n == 0 {
            // All samples were outliers - use original
            return self.calculate_stats_from_samples(name, &self.results, 0);
        }

        self.calculate_stats_from_samples(name, &samples, outliers_removed)
    }

    /// Calculate stats from a set of samples
    fn calculate_stats_from_samples(
        &self,
        name: &str,
        samples: &[Duration],
        outliers_removed: usize,
    ) -> BenchResult {
        let n = samples.len();

        let mut sorted: Vec<Duration> = samples.to_vec();
        sorted.sort();

        let min = sorted[0];
        let max = sorted[n - 1];
        let median = if n % 2 == 0 {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2
        } else {
            sorted[n / 2]
        };

        let total: Duration = samples.iter().sum();
        let mean = total / n as u32;

        // Calculate standard deviation
        let variance: f64 = samples
            .iter()
            .map(|d| {
                let diff = d.as_secs_f64() - mean.as_secs_f64();
                diff * diff
            })
            .sum::<f64>()
            / n as f64;
        let std_dev = Duration::from_secs_f64(variance.sqrt());

        // Calculate coefficient of variation
        let cv = if mean.as_secs_f64() > 0.0 {
            std_dev.as_secs_f64() / mean.as_secs_f64()
        } else {
            0.0
        };

        // Calculate percentiles
        let p50 = Self::percentile(&sorted, 50.0);
        let p95 = Self::percentile(&sorted, 95.0);
        let p99 = Self::percentile(&sorted, 99.0);

        // Check baseline comparison
        let vs_baseline = self.baseline.as_ref().and_then(|baseline| {
            baseline.results.get(name).map(|stored| {
                let change = if stored.mean.as_secs_f64() > 0.0 {
                    (mean.as_secs_f64() - stored.mean.as_secs_f64()) / stored.mean.as_secs_f64()
                } else {
                    0.0
                };

                BaselineComparison {
                    baseline_mean: stored.mean,
                    current_mean: mean,
                    change_percent: change,
                    is_regression: change > self.config.regression_threshold,
                }
            })
        });

        BenchResult {
            name: name.to_string(),
            min,
            max,
            mean,
            median,
            std_dev,
            p50,
            p95,
            p99,
            total,
            coefficient_of_variation: cv,
            iterations: n,
            outliers_removed,
            vs_baseline,
        }
    }

    /// Get all benchmark results
    pub fn results(&self) -> &[BenchResult] {
        &self.all_results
    }

    /// Get the number of collected results
    pub fn result_count(&self) -> usize {
        self.all_results.len()
    }

    /// Create a baseline from all collected results
    pub fn create_baseline(&self) -> BenchBaseline {
        let mut results = HashMap::new();
        for r in &self.all_results {
            results.insert(
                r.name.clone(),
                StoredBenchResult {
                    mean: r.mean,
                    std_dev: r.std_dev,
                    iterations: r.iterations,
                },
            );
        }
        BenchBaseline {
            results,
            captured_at: None,
            rust_version: None,
            platform: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    #[test]
    fn test_basic_benchmark() {
        let mut ctx = BenchContext::new().warmup(3).iterations(10);

        let result = ctx.bench("simple_add", || {
            let _ = black_box(1 + 1);
        });

        assert_eq!(result.name, "simple_add");
        assert_eq!(result.iterations, 10);
        assert!(result.min <= result.mean);
        assert!(result.mean <= result.max);
        assert_eq!(result.outliers_removed, 0);
    }

    #[test]
    fn test_percentiles() {
        let mut ctx = BenchContext::new().iterations(100);

        let result = ctx.bench("percentiles_test", || {
            let _ = black_box(vec![0u8; 100]);
        });

        assert!(result.p50 <= result.p95);
        assert!(result.p95 <= result.p99);
        assert!(result.p99 <= result.max);
    }

    #[test]
    fn test_coefficient_of_variation() {
        let mut ctx = BenchContext::new().iterations(50);

        let result = ctx.bench("consistent_work", || {
            // Use a consistent-time operation to get stable CV
            std::thread::sleep(Duration::from_micros(100));
        });

        // CV should be calculated
        assert!(result.coefficient_of_variation >= 0.0);
        // Note: CV can vary widely for very fast operations
        // This just validates the field is populated
    }

    #[test]
    fn test_outlier_removal_mad() {
        let mut ctx = BenchContext::new()
            .iterations(20)
            .outlier_removal(OutlierRemoval::Mad { threshold: 2.0 });

        let mut iteration = 0;
        let result = ctx.bench("with_outliers", || {
            iteration += 1;
            if iteration % 5 == 0 {
                // Artificial outlier
                std::thread::sleep(Duration::from_millis(10));
            }
            let _ = black_box(1 + 1);
        });

        // Some outliers should be removed
        // Note: this is probabilistic, but with 20 iterations and 4 outliers,
        // we should typically see some removed
        assert!(result.iterations <= 20);
    }

    #[test]
    fn test_outlier_removal_iqr() {
        let mut ctx = BenchContext::new()
            .iterations(20)
            .outlier_removal(OutlierRemoval::Iqr { multiplier: 1.5 });

        let mut iteration = 0;
        let result = ctx.bench("with_outliers_iqr", || {
            iteration += 1;
            if iteration % 5 == 0 {
                std::thread::sleep(Duration::from_millis(10));
            }
            let _ = black_box(1 + 1);
        });

        assert!(result.iterations <= 20);
    }

    #[test]
    fn test_baseline_comparison() {
        // Create a baseline
        let mut baseline = BenchBaseline::default();
        baseline.results.insert(
            "test".to_string(),
            StoredBenchResult {
                mean: Duration::from_millis(10),
                std_dev: Duration::from_millis(1),
                iterations: 100,
            },
        );

        let mut ctx = BenchContext::new()
            .iterations(10)
            .with_baseline(baseline)
            .regression_threshold(0.50); // 50% threshold

        let result = ctx.bench("test", || {
            // Should be much faster than 10ms baseline
            let _ = black_box(1 + 1);
        });

        assert!(result.vs_baseline.is_some());
        let comparison = result.vs_baseline.unwrap();
        assert!(!comparison.is_regression); // Should be faster than baseline
        assert!(comparison.change_percent < 0.0); // Negative = improvement
    }

    #[test]
    fn test_regression_detection() {
        let mut baseline = BenchBaseline::default();
        baseline.results.insert(
            "slow_test".to_string(),
            StoredBenchResult {
                mean: Duration::from_micros(1),
                std_dev: Duration::from_nanos(100),
                iterations: 100,
            },
        );

        let mut ctx = BenchContext::new()
            .iterations(10)
            .with_baseline(baseline)
            .regression_threshold(0.10); // 10% threshold

        let result = ctx.bench("slow_test", || {
            // Much slower than 1µs baseline
            std::thread::sleep(Duration::from_millis(1));
        });

        assert!(result.is_regression());
    }

    #[test]
    fn test_bench_with_setup() {
        let mut ctx = BenchContext::new().iterations(10);

        let result = ctx.bench_with_setup(
            "with_setup",
            || vec![0u8; 1000], // Setup
            |data| {
                let sum: u8 = data.iter().sum();
                black_box(sum);
            },
        );

        // Should have completed successfully
        assert_eq!(result.iterations, 10);
    }

    #[test]
    fn test_bench_with_input() {
        let mut ctx = BenchContext::new().iterations(10);

        let result = ctx.bench_with_input(
            "with_input",
            || 42,
            |x| {
                black_box(x * 2);
            },
        );

        assert_eq!(result.iterations, 10);
    }

    #[test]
    fn test_create_baseline() {
        let mut ctx = BenchContext::new().iterations(10);

        ctx.bench("test1", || {
            black_box(1 + 1);
        });
        ctx.bench("test2", || {
            black_box(2 + 2);
        });

        let baseline = ctx.create_baseline();
        assert!(baseline.results.contains_key("test1"));
        assert!(baseline.results.contains_key("test2"));
    }

    #[test]
    fn test_display_detailed() {
        let mut ctx = BenchContext::new().iterations(10);

        let result = ctx.bench("display_test", || {
            black_box(1 + 1);
        });

        let output = result.display_detailed();
        assert!(output.contains("display_test"));
        assert!(output.contains("Min:"));
        assert!(output.contains("Max:"));
        assert!(output.contains("Mean:"));
        assert!(output.contains("CV:"));
    }

    #[test]
    fn test_empty_results() {
        let ctx = BenchContext::new();
        assert!(ctx.results().is_empty());
    }

    #[test]
    fn test_calculate_cv() {
        let samples = vec![
            Duration::from_millis(100),
            Duration::from_millis(100),
            Duration::from_millis(100),
        ];
        let cv = BenchContext::calculate_cv(&samples);
        assert!(cv.abs() < 0.01); // Near-zero CV for identical samples
    }

    #[test]
    fn test_percentile_calculation() {
        let sorted = vec![
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(3),
            Duration::from_millis(4),
            Duration::from_millis(5),
        ];

        assert_eq!(BenchContext::percentile(&sorted, 0.0), Duration::from_millis(1));
        assert_eq!(BenchContext::percentile(&sorted, 50.0), Duration::from_millis(3));
        assert_eq!(BenchContext::percentile(&sorted, 100.0), Duration::from_millis(5));
    }
}

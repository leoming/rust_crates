use crate::parse_deqp::DeqpStatus;
use anyhow::{Context, Result};
use log::*;
use regex::Regex;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::str::FromStr;
use std::time::{Duration, Instant};
use structopt::StructOpt;

// Wrapper for displaying a duration in h:m:s (integer seconds, rounded down)
struct HMSDuration(Duration);
impl fmt::Display for HMSDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut secs = self.0.as_secs();
        let hours = secs / 3600;
        secs %= 3600;
        let mins = secs / 60;
        secs %= 60;

        if hours > 0 {
            write!(f, "{}:{:02}:{:02}", hours, mins, secs)
        } else if mins > 0 {
            write!(f, "{}:{:02}", mins, secs)
        } else {
            write!(f, "{}", secs)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunnerStatus {
    Pass,
    Fail,
    Skip,
    Crash,
    Flake,
    Warn,
    Missing,
    ExpectedFail,
    UnexpectedPass,
    Timeout,
}

impl FromStr for RunnerStatus {
    type Err = anyhow::Error;

    // Parses the status name from dEQP's output.
    fn from_str(input: &str) -> Result<RunnerStatus, Self::Err> {
        match input {
            "Pass" => Ok(RunnerStatus::Pass),
            "Fail" => Ok(RunnerStatus::Fail),
            "Crash" => Ok(RunnerStatus::Crash),
            "Skip" => Ok(RunnerStatus::Skip),
            "Flake" => Ok(RunnerStatus::Flake),
            "Warn" => Ok(RunnerStatus::Warn),
            "Missing" => Ok(RunnerStatus::Missing),
            "ExpectedFail" => Ok(RunnerStatus::ExpectedFail),
            "UnexpectedPass" => Ok(RunnerStatus::UnexpectedPass),
            "Timeout" => Ok(RunnerStatus::Timeout),
            _ => anyhow::bail!("unknown runner status '{}'", input),
        }
    }
}

impl RunnerStatus {
    pub fn is_success(&self) -> bool {
        match self {
            RunnerStatus::Pass
            | RunnerStatus::Skip
            | RunnerStatus::Warn
            | RunnerStatus::Flake
            | RunnerStatus::ExpectedFail => true,
            RunnerStatus::Fail
            | RunnerStatus::Crash
            | RunnerStatus::Missing
            | RunnerStatus::UnexpectedPass
            | RunnerStatus::Timeout => false,
        }
    }

    pub fn should_save_logs(&self, save_xfail_logs: bool) -> bool {
        !self.is_success()
            || *self == RunnerStatus::Flake
            || (*self == RunnerStatus::ExpectedFail && save_xfail_logs)
    }

    pub fn from_deqp(status: DeqpStatus) -> RunnerStatus {
        match status {
            DeqpStatus::Pass => RunnerStatus::Pass,
            DeqpStatus::Fail
            | DeqpStatus::ResourceError
            | DeqpStatus::InternalError
            | DeqpStatus::Pending => RunnerStatus::Fail,
            DeqpStatus::Crash => RunnerStatus::Crash,
            DeqpStatus::NotSupported => RunnerStatus::Skip,
            DeqpStatus::CompatibilityWarning | DeqpStatus::QualityWarning | DeqpStatus::Waiver => {
                RunnerStatus::Warn
            }
            DeqpStatus::Timeout => RunnerStatus::Timeout,
        }
    }

    pub fn with_baseline(self, baseline: Option<RunnerStatus>) -> RunnerStatus {
        use RunnerStatus::*;

        if let Some(baseline) = baseline {
            match self {
                Fail => match baseline {
                    Fail | ExpectedFail => ExpectedFail,
                    // This is a tricky one -- if you expected a crash and you got fail, that's
                    // an improvement where we should want to record the change in expectation,
                    // even though it's not properly a Pass.
                    Crash => UnexpectedPass,
                    // Ditto for timeouts
                    Timeout => UnexpectedPass,
                    _ => self,
                },
                Pass => match baseline {
                    Fail | Crash | Missing | Timeout => UnexpectedPass,
                    _ => Pass,
                },
                Crash => match baseline {
                    // If one is reusing a results.csv from a previous run with a baseline,
                    // then ExpectedFail might have been from a crash, so keep it as xfail.
                    Crash | ExpectedFail => ExpectedFail,
                    _ => Crash,
                },
                Warn => match baseline {
                    Fail | Crash => UnexpectedPass,
                    Warn => ExpectedFail,
                    _ => Warn,
                },
                Skip => {
                    // Should we report some state about tests going from Skip to Pass
                    // or vice versa?  The old runner didn't, so maintain that for now.
                    self
                }
                Flake => Flake,
                Timeout => match baseline {
                    Timeout => ExpectedFail,
                    _ => Timeout,
                },
                Missing | ExpectedFail | UnexpectedPass => {
                    unreachable!("can't appear from DeqpStatus")
                }
            }
        } else {
            self
        }
    }
}
impl fmt::Display for RunnerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            RunnerStatus::Pass => "Pass",
            RunnerStatus::Fail => "Fail",
            RunnerStatus::Skip => "Skip",
            RunnerStatus::Crash => "Crash",
            RunnerStatus::Warn => "Warn",
            RunnerStatus::Flake => "Flake",
            RunnerStatus::Missing => "Missing",
            RunnerStatus::ExpectedFail => "ExpectedFail",
            RunnerStatus::UnexpectedPass => "UnexpectedPass",
            RunnerStatus::Timeout => "Timeout",
        })
    }
}

#[derive(Debug)]
pub struct RunnerResult {
    pub test: String,
    pub status: RunnerStatus,
    pub duration: f32,
    pub subtest: bool,
}

// For comparing equality, we ignore the test runtime (particularly of use for the unit tests )
impl PartialEq for RunnerResult {
    fn eq(&self, other: &Self) -> bool {
        self.test == other.test && self.subtest == other.subtest && self.status == other.status
    }
}

pub struct RunnerResultNameHash(RunnerResult);

// Use the test name as the hash key, which lets us store a HashSet<RunnerResult> intead of HashMap<String,RunnerResult>.
impl core::hash::Hash for RunnerResultNameHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.test.hash(state);
    }
}

impl PartialEq for RunnerResultNameHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.test == other.0.test
    }
}

impl Eq for RunnerResultNameHash {}

impl Borrow<str> for RunnerResultNameHash {
    fn borrow(&self) -> &str {
        &self.0.test
    }
}

#[derive(Eq, Clone, Copy, Hash, PartialEq)]
pub struct CaselistState {
    pub caselist_id: u32,
    pub run_id: u32,
}

#[derive(Default, PartialEq, Debug)]
pub struct ResultCounts {
    pub pass: u32,
    pub fail: u32,
    pub skip: u32,
    pub crash: u32,
    pub warn: u32,
    pub flake: u32,
    pub missing: u32,
    pub expected_fail: u32,
    pub unexpected_pass: u32,
    pub timeout: u32,
    pub total: u32,
}
impl ResultCounts {
    pub fn new() -> ResultCounts {
        Default::default()
    }
    pub fn increment(&mut self, s: RunnerStatus) {
        match s {
            RunnerStatus::Pass => self.pass += 1,
            RunnerStatus::Fail => self.fail += 1,
            RunnerStatus::Skip => self.skip += 1,
            RunnerStatus::Crash => self.crash += 1,
            RunnerStatus::Warn => self.warn += 1,
            RunnerStatus::Flake => self.flake += 1,
            RunnerStatus::Missing => self.missing += 1,
            RunnerStatus::ExpectedFail => self.expected_fail += 1,
            RunnerStatus::UnexpectedPass => self.unexpected_pass += 1,
            RunnerStatus::Timeout => self.timeout += 1,
        }
    }

    pub fn get_count(&self, status: RunnerStatus) -> u32 {
        use RunnerStatus::*;

        match status {
            Pass => self.pass,
            Fail => self.fail,
            Skip => self.skip,
            Crash => self.crash,
            Warn => self.warn,
            Flake => self.flake,
            Missing => self.missing,
            ExpectedFail => self.expected_fail,
            UnexpectedPass => self.unexpected_pass,
            Timeout => self.timeout,
        }
    }
}
impl fmt::Display for ResultCounts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RunnerStatus::*;

        write!(f, "Pass: {}", self.pass)?;
        for status in &[
            Fail,
            Crash,
            UnexpectedPass,
            ExpectedFail,
            Warn,
            Skip,
            Timeout,
            Missing,
            Flake,
        ] {
            let count = self.get_count(*status);
            if count != 0 {
                write!(f, ", {}: {}", status, count)?;
            }
        }
        Ok(())
    }
}

pub struct RunnerResults {
    pub tests: HashSet<RunnerResultNameHash>,
    pub result_counts: ResultCounts,
    pub time: Instant,
}

impl RunnerResults {
    pub fn new() -> RunnerResults {
        Default::default()
    }

    pub fn get(&self, test: &str) -> Option<&RunnerResult> {
        self.tests.get(test).map(|x| &x.0)
    }

    pub fn record_result(&mut self, result: RunnerResult) {
        let mut result = result;

        if self.get(&result.test).is_some() {
            error!(
                "Duplicate test result for {}, marking test failed",
                &result.test
            );
            result.status = RunnerStatus::Fail;
        } else if !result.subtest {
            self.result_counts.total += 1;
        }
        self.result_counts.increment(result.status);
        self.tests.replace(RunnerResultNameHash(result));
    }

    pub fn is_success(&self) -> bool {
        self.tests.iter().all(|result| result.0.status.is_success())
    }

    /// Returns a list of references to the results, sorted by test name.
    pub fn sorted_results(&self) -> Vec<&RunnerResult> {
        let mut sorted: Vec<_> = self.tests.iter().map(|x| &x.0).collect();
        sorted.sort_by_key(|x| &x.test);
        sorted
    }

    pub fn write_results<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        for result in self.sorted_results() {
            writeln!(
                writer,
                "{},{},{}",
                result.test, result.status, result.duration
            )?;
        }
        Ok(())
    }

    pub fn write_failures(&self, writer: &mut impl Write) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        for result in self.sorted_results() {
            if !result.status.is_success() {
                writeln!(writer, "{},{}", result.test, result.status)?;
            }
        }
        Ok(())
    }

    pub fn write_junit_failures(
        &self,
        writer: &mut impl Write,
        options: &JunitGeneratorOptions,
    ) -> Result<()> {
        use junit_report::*;
        let limit = if options.limit == 0 {
            std::usize::MAX
        } else {
            options.limit
        };

        let mut testcases = Vec::new();
        for result in self.sorted_results().iter().take(limit) {
            let tc = if !result.status.is_success() {
                let message = options.template.replace("{{testcase}}", &result.test);

                let type_ = format!("{}", result.status);

                junit_report::TestCase::failure(
                    &result.test,
                    Duration::seconds(0),
                    &type_,
                    &message,
                )
            } else {
                junit_report::TestCase::success(&result.test, Duration::seconds(0))
            };
            testcases.push(tc);
        }

        let ts = junit_report::TestSuite::new(&options.testsuite).add_testcases(testcases);

        junit_report::Report::new()
            .add_testsuite(ts)
            .write_xml(BufWriter::new(writer))
            .context("writing XML output")
    }

    pub fn from_csv(r: &mut impl Read) -> Result<RunnerResults> {
        lazy_static! {
            static ref CSV_RE: Regex = Regex::new("^([^,]+),([^,]+)").unwrap();
        }

        let mut results = RunnerResults::new();
        let r = BufReader::new(r);
        for (lineno, line) in r.lines().enumerate() {
            let line = line.context("Reading CSV")?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(cap) = CSV_RE.captures(&line) {
                let result = RunnerResult {
                    test: cap[1].to_string(),
                    status: cap[2].parse()?,
                    duration: 0.0,
                    subtest: false,
                };

                // If you have more than one result for a test in the CSV,
                // something has gone wrong (probably human error writing a
                // baseline list)
                if results.get(&result.test).is_some() {
                    anyhow::bail!("Found duplicate result for {} at line {}", line, lineno);
                }

                results.record_result(result);
            } else {
                anyhow::bail!(
                    "Failed to parse {} as CSV test,status[,duration] or comment at line {}",
                    line,
                    lineno
                );
            }
        }
        Ok(results)
    }

    pub fn status_update<W: Write>(&self, writer: &mut W, total_tests: u32) {
        let duration = self.time.elapsed();

        write!(
            writer,
            "{}, Duration: {duration}",
            self.result_counts,
            duration = HMSDuration(duration),
        )
        .context("status update")
        .unwrap_or_else(|e| error!("{}", e));

        // If we have some tests completed, use that to estimate remaining runtime.
        let duration = duration.as_secs_f32();
        if self.result_counts.total != 0 {
            let average_test_time = duration / self.result_counts.total as f32;
            let remaining = average_test_time * (total_tests - self.result_counts.total) as f32;

            write!(
                writer,
                ", Remaining: {}",
                HMSDuration(Duration::from_secs_f32(remaining))
            )
            .context("status update")
            .unwrap_or_else(|e| error!("{}", e));
        }

        writeln!(writer)
            .context("status update")
            .unwrap_or_else(|e| error!("{}", e));
    }

    pub fn print_summary(&self, summary_limit: usize) {
        if self.tests.is_empty() {
            return;
        }

        let mut slowest: Vec<_> = self
            .tests
            .iter()
            .map(|result| (&result.0.test, result.0.duration))
            .collect();
        // Sorting on duration and reversing because you can't negate a duration and you can't easily
        // sort on a f32.
        slowest.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        slowest.reverse();

        println!();
        println!("Slowest tests:");
        for test in slowest.iter().take(5) {
            println!("  {} ({:.02}s)", test.0, test.1);
        }

        let mut flakes: Vec<_> = self
            .tests
            .iter()
            .map(|result| (&result.0.test, result.0.status))
            .filter(|(_, status)| *status == RunnerStatus::Flake)
            .collect();
        if !flakes.is_empty() {
            flakes.sort_by_key(|x| x.0);
            println!();
            println!("Some flaky tests found:");

            for test in flakes.iter().take(summary_limit) {
                println!("  {}", test.0)
            }
            if flakes.len() > summary_limit {
                println!("  ... and more (see results.csv)");
            }
        }

        let mut fails: Vec<_> = self
            .tests
            .iter()
            .map(|result| (&result.0.test, result.0.status))
            .filter(|(_, status)| !status.is_success())
            .collect();
        if !fails.is_empty() {
            fails.sort_by_key(|x| x.0);
            println!();
            println!("Some failures found:");

            for test in fails.iter().take(summary_limit) {
                println!("  {},{}", test.0, test.1)
            }
            if fails.len() > summary_limit {
                println!("  ... and more (see failures.csv)");
            }
        }
    }
}

impl Default for RunnerResults {
    fn default() -> RunnerResults {
        RunnerResults {
            tests: Default::default(),
            result_counts: Default::default(),
            time: Instant::now(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_map() {
        use RunnerStatus::*;

        // Absence of baseline should translate in a straightforward way.
        assert_eq!(
            Pass,
            RunnerStatus::from_deqp(DeqpStatus::Pass).with_baseline(None)
        );
        assert_eq!(
            Fail,
            RunnerStatus::from_deqp(DeqpStatus::Fail).with_baseline(None)
        );
        assert_eq!(
            Crash,
            RunnerStatus::from_deqp(DeqpStatus::Crash).with_baseline(None)
        );
        assert_eq!(
            Warn,
            RunnerStatus::from_deqp(DeqpStatus::CompatibilityWarning).with_baseline(None)
        );
        assert_eq!(
            Warn,
            RunnerStatus::from_deqp(DeqpStatus::QualityWarning).with_baseline(None)
        );

        // Basic expected failures handling.
        assert_eq!(
            ExpectedFail,
            RunnerStatus::from_deqp(DeqpStatus::Fail).with_baseline(Some(Fail))
        );
        assert_eq!(
            ExpectedFail,
            RunnerStatus::from_deqp(DeqpStatus::Crash).with_baseline(Some(Crash))
        );
        assert_eq!(
            ExpectedFail,
            RunnerStatus::from_deqp(DeqpStatus::CompatibilityWarning).with_baseline(Some(Warn))
        );
        assert_eq!(
            ExpectedFail,
            RunnerStatus::from_deqp(DeqpStatus::Timeout).with_baseline(Some(Timeout))
        );
        assert_eq!(
            UnexpectedPass,
            RunnerStatus::from_deqp(DeqpStatus::Pass).with_baseline(Some(Fail))
        );

        assert_eq!(
            UnexpectedPass,
            RunnerStatus::from_deqp(DeqpStatus::Fail).with_baseline(Some(Crash))
        );
        assert_eq!(
            UnexpectedPass,
            RunnerStatus::from_deqp(DeqpStatus::Fail).with_baseline(Some(Timeout))
        );
        assert_eq!(
            UnexpectedPass,
            RunnerStatus::from_deqp(DeqpStatus::Pass).with_baseline(Some(Timeout))
        );

        // Should be able to fee a run with a baseline as a new baseline (though you lose some Crash details)
        assert_eq!(
            ExpectedFail,
            RunnerStatus::from_deqp(DeqpStatus::Fail).with_baseline(Some(ExpectedFail))
        );
        assert_eq!(
            ExpectedFail,
            RunnerStatus::from_deqp(DeqpStatus::Crash).with_baseline(Some(ExpectedFail))
        );

        // If we expected this internal runner error and it stops happening, time to update expectaions
        assert_eq!(
            UnexpectedPass,
            RunnerStatus::from_deqp(DeqpStatus::Pass).with_baseline(Some(Missing))
        );
    }

    #[test]
    fn csv_parse() -> Result<()> {
        let results = RunnerResults::from_csv(
            &mut "
# This is a comment in a baseline CSV file, with a comma, to make sure we skip them.

dEQP-GLES2.info.version,Fail
piglit@crashy@test,Crash"
                .as_bytes(),
        )?;
        assert_eq!(
            results.get("dEQP-GLES2.info.version"),
            Some(RunnerResult {
                test: "dEQP-GLES2.info.version".to_string(),
                status: RunnerStatus::Fail,
                duration: 0.0,
                subtest: false,
            })
            .as_ref()
        );
        assert_eq!(
            results.get("piglit@crashy@test"),
            Some(RunnerResult {
                test: "piglit@crashy@test".to_string(),
                status: RunnerStatus::Crash,
                duration: 0.0,
                subtest: false,
            })
            .as_ref()
        );

        Ok(())
    }

    #[test]
    fn csv_parse_dup() {
        assert!(RunnerResults::from_csv(
            &mut "
dEQP-GLES2.info.version,Fail
dEQP-GLES2.info.version,Pass"
                .as_bytes(),
        )
        .is_err());
    }

    #[test]
    #[allow(clippy::string_lit_as_bytes)]
    fn csv_test_missing_status() {
        assert!(RunnerResults::from_csv(&mut "dEQP-GLES2.info.version".as_bytes()).is_err());
    }

    #[test]
    fn hms_display() {
        assert_eq!(format!("{}", HMSDuration(Duration::new(15, 20))), "15");
        assert_eq!(format!("{}", HMSDuration(Duration::new(0, 20))), "0");
        assert_eq!(format!("{}", HMSDuration(Duration::new(70, 20))), "1:10");
        assert_eq!(format!("{}", HMSDuration(Duration::new(69, 20))), "1:09");
        assert_eq!(
            format!("{}", HMSDuration(Duration::new(60 * 60 + 3, 20))),
            "1:00:03"
        );
        assert_eq!(
            format!("{}", HMSDuration(Duration::new(3735, 20))),
            "1:02:15"
        );
    }

    fn add_result(results: &mut RunnerResults, test: &str, status: RunnerStatus) {
        results.record_result(RunnerResult {
            test: test.to_string(),
            status,
            duration: 0.0,
            subtest: false,
        });
    }

    #[test]
    fn results_is_success() {
        let mut results = RunnerResults::new();

        add_result(&mut results, "pass1", RunnerStatus::Pass);
        add_result(&mut results, "pass2", RunnerStatus::Pass);

        assert!(results.is_success());

        add_result(&mut results, "Crash", RunnerStatus::Crash);

        assert!(!results.is_success());
    }
}

#[derive(Debug, StructOpt)]
pub struct JunitGeneratorOptions {
    #[structopt(long, help = "Testsuite name for junit")]
    testsuite: String,

    #[structopt(
        long,
        default_value = "",
        help = "Failure message template (with {{testcase}} replaced with the test name)"
    )]
    template: String,

    #[structopt(
        long,
        default_value = "0",
        help = "Number of junit cases to list (or 0 for unlimited)"
    )]
    limit: usize,
}

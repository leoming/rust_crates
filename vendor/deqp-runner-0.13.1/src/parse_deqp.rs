use anyhow::{Context, Result};
use regex::Regex;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::time::{Duration, Instant};
use timeout_readwrite::TimeoutReader;

// See s_qpTestResultMap in qpTestLog.c
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeqpStatus {
    Pass,
    Fail,
    QualityWarning,
    CompatibilityWarning,
    Pending,
    NotSupported,
    ResourceError,
    InternalError,
    Crash,
    Timeout,
    Waiver,
}

impl FromStr for DeqpStatus {
    type Err = anyhow::Error;

    // Parses the status name from dEQP's output.
    fn from_str(input: &str) -> Result<DeqpStatus, Self::Err> {
        match input {
            "Pass" => Ok(DeqpStatus::Pass),
            "Fail" => Ok(DeqpStatus::Fail),
            "QualityWarning" => Ok(DeqpStatus::QualityWarning),
            "CompatibilityWarning" => Ok(DeqpStatus::CompatibilityWarning),
            "Pending" => Ok(DeqpStatus::Pending),
            "NotSupported" => Ok(DeqpStatus::NotSupported),
            "ResourceError" => Ok(DeqpStatus::ResourceError),
            "InternalError" => Ok(DeqpStatus::InternalError),
            "Crash" => Ok(DeqpStatus::Crash),
            "Timeout" => Ok(DeqpStatus::Timeout),
            "Waiver" => Ok(DeqpStatus::Waiver),
            _ => anyhow::bail!("unknown dEQP status '{}'", input),
        }
    }
}

#[derive(Debug)]
pub struct DeqpTestResult {
    pub name: String,
    pub status: DeqpStatus,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct DeqpCaselistResult {
    pub results: Vec<DeqpTestResult>,
    pub stdout: Vec<String>,
}

// For comparing equality, we ignore the test runtime (particularly of use for the unit tests )
impl PartialEq for DeqpTestResult {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.status == other.status
    }
}

pub fn parse_deqp_results(deqp_output: impl Read) -> Result<DeqpCaselistResult> {
    let deqp_output = BufReader::new(deqp_output);
    let mut current_test: Option<(String, Instant)> = None;
    lazy_static! {
        static ref TEST_RE: Regex = Regex::new("Test case '(.*)'..").unwrap();
        static ref STATUS_RE: Regex = Regex::new("^  (\\S*) \\(.*\\)").unwrap();
    }

    let mut stdout: Vec<String> = Vec::new();
    let mut results = Vec::new();

    for line in deqp_output.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                if let std::io::ErrorKind::TimedOut = e.kind() {
                    if let Some((ref name, time)) = current_test {
                        results.push(DeqpTestResult {
                            name: name.to_string(),
                            status: DeqpStatus::Timeout,
                            duration: time.elapsed(),
                        });
                        return Ok(DeqpCaselistResult { results, stdout });
                    }
                }

                return Err(e).context("Reading from dEQP");
            }
        };

        stdout.push(line);
        let line = stdout.last().unwrap();

        if let Some((ref name, time)) = current_test {
            if let Some(cap) = STATUS_RE.captures(line) {
                let status = &cap[1];

                /* One of the VK sparse tests emits some info lines about a NotSupported before actually saying NotSupported. */
                if status == "Info" {
                    continue;
                }

                let status = DeqpStatus::from_str(&cap[1])?;

                results.push(DeqpTestResult {
                    name: name.to_string(),
                    status,
                    duration: time.elapsed(),
                });
                current_test = None;
            }
        } else if let Some(cap) = TEST_RE.captures(line) {
            current_test = Some((cap[1].to_string(), Instant::now()));
        }
    }
    if let Some((ref name, time)) = current_test {
        results.push(DeqpTestResult {
            name: name.clone(),
            status: DeqpStatus::Crash,
            duration: time.elapsed(),
        });
    }

    Ok(DeqpCaselistResult { results, stdout })
}

pub fn parse_deqp_results_with_timeout(
    deqp_output: impl Read + std::os::unix::io::AsRawFd,
    timeout: Duration,
) -> Result<DeqpCaselistResult> {
    parse_deqp_results(TimeoutReader::new(deqp_output, timeout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    fn result(name: &str, status: DeqpStatus) -> DeqpTestResult {
        DeqpTestResult {
            name: name.to_string(),
            status,
            duration: Duration::new(0, 0),
        }
    }

    #[test]
    fn parse_statuses() {
        let output = "
Writing test log into /home/anholt/TestResults.qpa
dEQP Core git-f442587f49bded2eec0c04fca6eb78d60ee83e8f (0xf442587f) starting..
  target implementation = 'Surfaceless'

Test case 'dEQP-GLES2.test.p'..
  Pass (Supported)

Test case 'dEQP-GLES2.test.f'..
  Fail (Found invalid pixel values)

Test case 'dEQP-GLES2.test.q'..
  QualityWarning (not so hot)

Test case 'dEQP-GLES2.test.c'..
  CompatibilityWarning (some bug)

Test case 'dEQP-GLES2.test.pend'..
  Pending (is there even a test that emits this)

Test case 'dEQP-GLES2.test.ns'..
  NotSupported (GL_EXT_tessellation_shader is not supported)

Test case 'dEQP-GLES2.test.re'..
  ResourceError (something missing)

Test case 'dEQP-GLES2.test.ie'..
  InternalError (whoops)

Test case 'dEQP-GLES2.test.cr'..
  Crash (boom)

Test case 'dEQP-GLES2.test.time'..
  Timeout (deqp watchdog)

Test case 'dEQP-GLES2.test.waive'..
  Waiver (it's fine)

DONE!

Test run totals:
  Passed:        6/6 (100.0%)
  Failed:        0/6 (0.0%)
  Not supported: 0/6 (0.0%)
  Warnings:      0/6 (0.0%)
  Waived:        0/6 (0.0%)";

        assert_eq!(
            parse_deqp_results(&mut output.as_bytes()).unwrap().results,
            vec![
                result("dEQP-GLES2.test.p", DeqpStatus::Pass),
                result("dEQP-GLES2.test.f", DeqpStatus::Fail),
                result("dEQP-GLES2.test.q", DeqpStatus::QualityWarning),
                result("dEQP-GLES2.test.c", DeqpStatus::CompatibilityWarning),
                result("dEQP-GLES2.test.pend", DeqpStatus::Pending),
                result("dEQP-GLES2.test.ns", DeqpStatus::NotSupported),
                result("dEQP-GLES2.test.re", DeqpStatus::ResourceError),
                result("dEQP-GLES2.test.ie", DeqpStatus::InternalError),
                result("dEQP-GLES2.test.cr", DeqpStatus::Crash),
                result("dEQP-GLES2.test.time", DeqpStatus::Timeout),
                result("dEQP-GLES2.test.waive", DeqpStatus::Waiver),
            ]
        );
    }

    #[test]
    /// Test parsing a run that didn't produce any results, common when doing something like
    /// using a test list for the wrong deqp binary.
    fn parse_all_missing() {
        let output = "
dEQP Core git-f442587f49bded2eec0c04fca6eb78d60ee83e8f (0xf442587f) starting..
  target implementation = 'Surfaceless'

DONE!

Test run totals:
  Passed:        0/0 (0.0%)
  Failed:        0/0 (0.0%)
  Not supported: 0/0 (0.0%)
  Warnings:      0/0 (0.0%)
  Waived:        0/0 (0.0%)";

        assert_eq!(
            parse_deqp_results(&mut output.as_bytes()).unwrap().results,
            Vec::new()
        );
    }
    #[test]
    fn parse_crash() {
        let output = "
Writing test log into /home/anholt/TestResults.qpa
dEQP Core git-f442587f49bded2eec0c04fca6eb78d60ee83e8f (0xf442587f) starting..
  target implementation = 'Surfaceless'

Test case 'dEQP-GLES2.test.p'..
  Pass (Supported)

Test case 'dEQP-GLES2.test.c'..";

        assert_eq!(
            parse_deqp_results(&mut output.as_bytes()).unwrap().results,
            vec![
                result("dEQP-GLES2.test.p", DeqpStatus::Pass),
                result("dEQP-GLES2.test.c", DeqpStatus::Crash),
            ]
        );
    }

    #[test]
    fn parse_failure() {
        let output = "
Writing test log into /home/anholt/TestResults.qpa
dEQP Core git-f442587f49bded2eec0c04fca6eb78d60ee83e8f (0xf442587f) starting..
  target implementation = 'Surfaceless'

Test case 'dEQP-GLES2.test.p'..
  Pass (Supported)

Test case 'dEQP-GLES2.test.parsefail'..
  UnknownStatus (unknown)";

        assert!(parse_deqp_results(&mut output.as_bytes()).is_err());
    }

    #[test]
    fn parse_parens_in_detail() {
        let output = "
Writing test log into /home/anholt/TestResults.qpa
dEQP Core git-f442587f49bded2eec0c04fca6eb78d60ee83e8f (0xf442587f) starting..
  target implementation = 'Surfaceless'

Test case 'dEQP-GLES2.test.parens'..
  NotSupported (Test requires GL_MAX_VERTEX_SHADER_STORAGE_BLOCKS (0) >= 0)'..";

        assert_eq!(
            parse_deqp_results(&mut output.as_bytes()).unwrap().results,
            vec![result("dEQP-GLES2.test.parens", DeqpStatus::NotSupported),]
        );
    }

    #[test]
    fn parse_timeout() -> Result<()> {
        // The spawning here mimics DeqpCommand's, since we want to make sure that it
        // will handle timeouts well, but we don't really want all of its deqp temp files
        // handling for this test.
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = std::io::BufWriter::new(child.stdin.take().unwrap());

        stdin
            .write_all(
                "
dEQP Core git-f442587f49bded2eec0c04fca6eb78d60ee83e8f (0xf442587f) starting..
    target implementation = 'Surfaceless'
Test case 'dEQP-GLES2.test.p'..
  Pass (Supported)

Test case 'dEQP-GLES2.test.t'..
"
                .as_bytes(),
            )
            .unwrap();
        stdin.flush().unwrap();

        let stdout = child.stdout.take().context("opening stdout")?;

        assert_eq!(
            parse_deqp_results_with_timeout(stdout, Duration::new(0, 100_000_000))
                .context("parsing results")
                .unwrap()
                .results,
            vec![
                result("dEQP-GLES2.test.p", DeqpStatus::Pass),
                result("dEQP-GLES2.test.t", DeqpStatus::Timeout),
            ]
        );

        child.kill().context("killing cat")
    }

    #[test]
    fn parse_turnip_stdout_mixup() -> Result<()> {
        let output = include_bytes!("../resources/test/turnip-stdout-mixup.txt");
        let caselist_results = parse_deqp_results(&mut output.as_ref())?;
        let results = caselist_results.results;

        for result in &results {
            // Make sure parsing didn't mix up our stdout with some other lines.
            assert!(result.name.starts_with("dEQP-VK."));
        }

        assert_eq!(
            results[results.len() - 1].name,
            "dEQP-VK.compute.device_group.device_index"
        );
        assert_eq!(results[results.len() - 1].status, DeqpStatus::Crash);

        assert_eq!(results.len(), 347);

        Ok(())
    }

    #[test]
    fn parse_vk_sparse_info() -> Result<()> {
        let output = r#"
Test case 'dEQP-VK.api.buffer_memory_requirements.create_sparse_binding_sparse_residency_sparse_aliased.ext_mem_flags_included.method1.size_req_transfer_usage_bits'..
  Info (Create buffer with VK_BUFFER_CREATE_SPARSE_BINDING_BIT not supported by device at vktApiBufferMemoryRequirementsTests.cpp:353)
  Info (Create buffer with VK_BUFFER_CREATE_SPARSE_RESIDENCY_BIT not supported by device at vktApiBufferMemoryRequirementsTests.cpp:359)
  Info (Create buffer with VK_BUFFER_CREATE_SPARSE_ALIASED_BIT not supported by device at vktApiBufferMemoryRequirementsTests.cpp:365)
  NotSupported (One or more create buffer flags not supported by device at vktApiBufferMemoryRequirementsTests.cpp:377)"#;

        let caselist_results = parse_deqp_results(&mut output.as_bytes())?;
        let results = caselist_results.results;
        assert_eq!(results[0].status, DeqpStatus::NotSupported);

        Ok(())
    }
}

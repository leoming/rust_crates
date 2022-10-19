use crate::parse_deqp::{DeqpStatus, DeqpTestResult};
use crate::runner_results::*;
use crate::{runner_thread_index, TestCase, TestCommand, TestConfiguration};
use anyhow::{Context, Result};
use log::*;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use timeout_readwrite::TimeoutReader;

pub struct GTestCommand {
    pub bin: PathBuf,
    pub config: TestConfiguration,
    pub args: Vec<String>,
}

impl DeqpStatus {
    pub fn from_gtest_str(input: &str) -> Result<DeqpStatus> {
        match input {
            "OK" => Ok(DeqpStatus::Pass),
            "FAILED" => Ok(DeqpStatus::Fail),
            "SKIPPED" => Ok(DeqpStatus::NotSupported),
            _ => anyhow::bail!("unknown gtest status '{}'", input),
        }
    }
}

pub fn parse_test_list(input: &str) -> Result<Vec<TestCase>> {
    let group_regex = Regex::new(r#"(\S*\.)"#).context("Compiling group RE")?;
    let test_regex = Regex::new(r#"\s+(\S+)"#).context("Compiling test RE")?;

    let mut tests = Vec::new();

    let mut group = None;
    for line in input.lines() {
        if let Some(c) = group_regex.captures(line) {
            group = Some(c[1].to_owned());
        } else if let Some(c) = test_regex.captures(line) {
            if let Some(group) = &group {
                tests.push(TestCase::GTest(format!("{}{}", group, &c[1])));
            } else {
                anyhow::bail!("Test with no group set at: '{}'", line);
            }
        } else {
            anyhow::bail!("Failed to parse gtest list output: '{}'", line);
        }
    }

    Ok(tests)
}

#[derive(Default)]
pub struct GTestResults {
    results: Vec<DeqpTestResult>,
    stdout: Vec<String>,
}

impl GTestResults {
    pub fn new() -> GTestResults {
        GTestResults::default()
    }
}

pub fn parse_gtest_results(gtest_output: impl Read) -> Result<GTestResults> {
    let gtest_output = BufReader::new(gtest_output);
    let mut current_test: Option<(String, Instant)> = None;
    lazy_static! {
        static ref TEST_RE: Regex = Regex::new(r#"\[\sRUN\s*\]\s(.*)"#).unwrap();
        static ref STATUS_RE: Regex = Regex::new(r#"\[\s*(\S*)\s*\]\s(\S*)"#).unwrap();
    }

    let mut results = GTestResults::new();

    for line in gtest_output.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                if let std::io::ErrorKind::TimedOut = e.kind() {
                    if let Some((ref name, time)) = current_test {
                        results.results.push(DeqpTestResult {
                            name: name.to_string(),
                            status: DeqpStatus::Timeout,
                            duration: time.elapsed(),
                        });
                        return Ok(results);
                    }
                }

                return Err(e).context("Reading from gtest");
            }
        };

        results.stdout.push(line.clone());

        if let Some((ref name, time)) = current_test {
            if let Some(cap) = STATUS_RE.captures(&line) {
                let status = DeqpStatus::from_gtest_str(&cap[1])?;

                results.results.push(DeqpTestResult {
                    name: name.to_string(),
                    status,
                    duration: time.elapsed(),
                });
                current_test = None;
            }
        } else if let Some(cap) = TEST_RE.captures(&line) {
            current_test = Some((cap[1].to_string(), Instant::now()));
        }
    }
    if let Some((ref name, time)) = current_test {
        results.results.push(DeqpTestResult {
            name: name.clone(),
            status: DeqpStatus::Crash,
            duration: time.elapsed(),
        });
    }

    Ok(results)
}

pub fn parse_gtest_results_with_timeout(
    gtest_output: impl Read + std::os::unix::io::AsRawFd,
    timeout: Duration,
) -> Result<GTestResults> {
    parse_gtest_results(TimeoutReader::new(gtest_output, timeout))
}

impl GTestCommand {
    pub fn list_tests(&self) -> Result<Vec<TestCase>> {
        let output = Command::new(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .args(&self.args)
            .arg("--gtest_list_tests")
            .output()
            .with_context(|| format!("Failed to spawn {}", &self.bin.display()))?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to invoke gtest command {} for test listing:\nstdout:\n{}\nstderr:\n{}",
                self.bin.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        parse_test_list(
            std::str::from_utf8(&output.stdout).context("Parsing gtest output as UTF8")?,
        )
    }
}

impl TestCommand for GTestCommand {
    fn run(
        &self,
        caselist_state: &CaselistState,
        tests: &[&TestCase],
    ) -> Result<Vec<RunnerResult>> {
        let mut tests_iter = tests.iter();

        let mut tests_arg = format!(
            "--gtest_filter={}",
            tests_iter.next().context("getting first test")?.name()
        );

        for test in tests_iter {
            tests_arg.push(':');
            tests_arg.push_str(test.name());
        }

        let mut command = Command::new(&self.bin);
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .env("DEQP_RUNNER_THREAD", runner_thread_index()?.to_string())
            .args(&self.args)
            .arg(tests_arg);

        let command_line = format!("{:?}", command);

        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to spawn {}", &self.bin.display()))?;

        let stdout = child.stdout.take().context("opening stdout")?;
        let gtest_results = parse_gtest_results_with_timeout(stdout, self.config.timeout);

        // The child should have run to completion based on parse_gtest_results() consuming its output,
        // but if we had a timeout or parse failure then we want to kill this run.
        let _ = child.kill();

        // Make sure we reap the child process.
        let status = child.wait().context("waiting for child")?;

        let GTestResults {
            results: mut gtest_results,
            stdout,
        } = gtest_results.context("parsing results")?;

        let stderr: Vec<String> = BufReader::new(child.stderr.as_mut().context("opening stderr")?)
            .lines()
            .flatten()
            .collect();

        for line in &stderr {
            // If the driver has ASan enabled and it detected leaks, then mark
            // all the tests in the caselist as failed (since we don't know who
            // to assign the failure to).
            if line.contains("ERROR: LeakSanitizer: detected memory leaks") {
                error!(
                    "gtest-runner: Leak detected, marking caselist as failed ({})",
                    self.see_more("", caselist_state)
                );
                for result in gtest_results.iter_mut() {
                    result.status = DeqpStatus::Fail;
                }
            }
            error!("gtest error: {}", line);
        }

        let mut save_log = false;
        let mut results: Vec<RunnerResult> = Vec::new();
        for result in gtest_results {
            let status = self.translate_result(&result, caselist_state);

            if status.should_save_logs(self.config.save_xfail_logs) {
                save_log = true;
            }

            results.push(RunnerResult {
                test: result.name,
                status,
                duration: result.duration.as_secs_f32(),
                subtest: false,
            });
        }

        if save_log {
            let log_path = self
                .caselist_file_path(caselist_state, "log")
                .context("log path")?;

            let mut file = File::create(log_path).context("opening log file")?;

            fn write_output(file: &mut File, name: &str, out: &[String]) -> Result<()> {
                if out.is_empty() {
                    writeln!(file, "{}: (empty)", name)?;
                } else {
                    writeln!(file, "{}:", name)?;
                    writeln!(file, "-------")?;
                    for line in out {
                        writeln!(file, "{}", line)?;
                    }
                }
                Ok(())
            }

            // Use a closure to wrap all the try operator paths with one .context().
            || -> Result<()> {
                writeln!(file, "command: {}", command_line)?;
                writeln!(file, "exit status: {}", status)?;
                write_output(&mut file, "stdout", &stdout)?;
                write_output(&mut file, "stderr", &stderr)?;
                Ok(())
            }()
            .context("writing log file")?;
        }

        Ok(results)
    }

    fn see_more(&self, _name: &str, caselist_state: &CaselistState) -> String {
        // This is the same as run() did, so we should be safe to unwrap.
        let qpa_path = self.config.output_dir.join(
            format!(
                "c{}.r{}.log",
                caselist_state.caselist_id, caselist_state.run_id
            )
            .as_str(),
        );
        format!("See {:?}", qpa_path)
    }

    fn config(&self) -> &TestConfiguration {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(name: &str, status: DeqpStatus) -> DeqpTestResult {
        DeqpTestResult {
            name: name.to_string(),
            status,
            duration: Duration::new(0, 0),
        }
    }

    #[test]
    fn list_tests() -> Result<()> {
        let input = "VAAPICreateContextToFail.
  CreateContextWithNoConfig
VAAPIDisplayAttribs.
  MaxNumDisplayAttribs
  QueryDisplayAttribs
GetCreateConfig/VAAPIGetCreateConfig.
  CreateConfigWithAttributes/0  # GetParam() = (-1:VAProfileNone, 1:VAEntrypointVLD)
  CreateConfigWithAttributes/1  # GetParam() = (-1:VAProfileNone, 2:VAEntrypointIZZ)";

        let tests: Vec<String> = parse_test_list(input)
            .context("parsing")?
            .into_iter()
            .map(|x| x.name().to_string())
            .collect();

        assert_eq!(
            tests,
            vec!(
                "VAAPICreateContextToFail.CreateContextWithNoConfig".to_owned(),
                "VAAPIDisplayAttribs.MaxNumDisplayAttribs".to_owned(),
                "VAAPIDisplayAttribs.QueryDisplayAttribs".to_owned(),
                "GetCreateConfig/VAAPIGetCreateConfig.CreateConfigWithAttributes/0".to_owned(),
                "GetCreateConfig/VAAPIGetCreateConfig.CreateConfigWithAttributes/1".to_owned(),
            )
        );
        Ok(())
    }

    #[test]
    fn parse_results() -> Result<()> {
        use DeqpStatus::*;
        let output = "[----------] 1 test from VAAPIQueryVendor
[ RUN      ] VAAPIQueryVendor.NotEmpty
[       OK ] VAAPIQueryVendor.NotEmpty (11 ms)
[----------] 1 test from VAAPIQueryVendor (11 ms total)

[----------] 690 tests from GetCreateConfig/VAAPIGetCreateConfig
[ RUN      ] GetCreateConfig/VAAPIGetCreateConfig.CreateConfigNoAttributes/219
../test/test_va_api_fixture.cpp:224: Failure
      Expected: VaapiStatus(expectation)
      Which is: VA_STATUS_ERROR_UNSUPPORTED_PROFILE
To be equal to: VaapiStatus(vaCreateConfig(m_vaDisplay, profile, entrypoint, (attribs.size() != 0 ? const_cast<VAConfigAttrib*>(attribs.data()) : __null), attribs.size(), &m_configID))
      Which is: VA_STATUS_ERROR_UNSUPPORTED_ENTRYPOINT
profile    = 21:VAProfileVP9Profile2
entrypoint = 11:VAEntrypointFEI
numAttribs = 0
[  FAILED  ] GetCreateConfig/VAAPIGetCreateConfig.CreateConfigNoAttributes/219, where GetParam() = (21:VAProfileVP9Profile2, 11:VAEntrypointFEI) (11 ms)
[ RUN      ] GetCreateConfig/VAAPIGetCreateConfig.CreateConfigPackedHeaders/0
[ SKIPPED ] -1:VAProfileNone / 1:VAEntrypointVLD not supported on this hardware
[       OK ] GetCreateConfig/VAAPIGetCreateConfig.CreateConfigPackedHeaders/0
[ RUN      ] CreateSurfaces/VAAPICreateSurfaces.CreateSurfacesWithConfigAttribs/136";

        let results = parse_gtest_results(&mut output.as_bytes())?.results;

        assert_eq!(
            results,
            vec!(
                result("VAAPIQueryVendor.NotEmpty", Pass),
                result(
                    "GetCreateConfig/VAAPIGetCreateConfig.CreateConfigNoAttributes/219",
                    Fail
                ),
                result(
                    "GetCreateConfig/VAAPIGetCreateConfig.CreateConfigPackedHeaders/0",
                    NotSupported
                ),
                result(
                    "CreateSurfaces/VAAPICreateSurfaces.CreateSurfacesWithConfigAttribs/136",
                    Crash
                ),
            )
        );

        Ok(())
    }
}

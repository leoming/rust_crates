use crate::parse_deqp::{DeqpStatus, DeqpTestResult};
use crate::parse_piglit::{
    parse_piglit_results_with_timeout, piglit_sanitize_test_name, PiglitTestResult,
};
use crate::parse_piglit::{parse_piglit_xml_testlist, read_profile_file};
use crate::{
    parse_regex_set, runner_results::*, runner_thread_index, SubRunConfig, TestConfiguration,
};
use crate::{TestCase, TestCommand};
use anyhow::{Context, Result};
use log::*;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use structopt::StructOpt;

pub struct PiglitCommand {
    pub config: TestConfiguration,
    pub piglit_folder: PathBuf,
    pub prefix: String,
}

// Common structure for configuring a piglit run between Run (single run) and deqp-runner Suite (muliple Runs)
#[derive(Debug, Deserialize, StructOpt)]
pub struct PiglitRunConfig {
    #[structopt(long, help = "path to piglit folder")]
    pub piglit_folder: PathBuf,

    #[structopt(long, help = "piglit profile to run (such as quick_gl)")]
    pub profile: String,

    #[structopt(long = "process-isolation")]
    #[serde(default)]
    pub process_isolation: bool,
}

#[derive(Deserialize)]
pub struct PiglitTomlConfig {
    #[serde(flatten)]
    pub sub_config: SubRunConfig,

    #[serde(flatten)]
    pub piglit_config: PiglitRunConfig,

    #[serde(default)]
    pub prefix: String,
}

impl PiglitTomlConfig {
    pub fn test_groups<'d>(
        &self,
        piglit: &'d PiglitCommand,
        filters: &[String],
    ) -> Result<Vec<(&'d dyn TestCommand, Vec<TestCase>)>> {
        let mut include_filters = Vec::new();
        if !self.sub_config.include.is_empty() {
            include_filters.push(
                parse_regex_set(&self.sub_config.include).context("compiling include filters")?,
            );
        }
        if !filters.is_empty() {
            include_filters.push(parse_regex_set(filters).context("compiling include filters")?);
        }

        let test_folder = self.piglit_config.piglit_folder.join("tests");
        let text = read_profile_file(
            &test_folder,
            &self.piglit_config.profile,
            self.piglit_config.process_isolation,
        )?;
        let tests: Vec<TestCase> =
            parse_piglit_xml_testlist(&test_folder, &text, self.piglit_config.process_isolation)
                .with_context(|| {
                    format!("reading piglit profile '{}'", &self.piglit_config.profile)
                })?
                .into_iter()
                .skip(self.sub_config.fraction_start - 1)
                .step_by(self.sub_config.fraction)
                .filter(|test| include_filters.iter().all(|x| x.is_match(test.name())))
                .collect();

        println!(
            "Running {} piglit tests on {} threads",
            tests.len(),
            rayon::current_num_threads()
        );

        let groups = piglit.split_tests_to_groups(tests, 1, 1)?;

        Ok(groups)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PiglitTest {
    pub name: String,
    pub binary: String,
    pub args: Vec<String>,
}

impl TestCommand for PiglitCommand {
    fn run(
        &self,
        caselist_state: &CaselistState,
        tests: &[&TestCase],
    ) -> Result<Vec<RunnerResult>> {
        let mut bin_path = self.piglit_folder.clone();
        bin_path.push("bin");

        // We only run one piglit command in a test group.  This means that
        // flake detection doesn't have to run irrelevant tests, and makes our
        // log file handling easier.
        assert_eq!(tests.len(), 1);
        let test = &tests[0];

        let test = match test {
            TestCase::Piglit(t) => t,
            _ => panic!("Invalid case"),
        };

        let log_path = self
            .config
            .output_dir
            .join(format!("piglit.{}.log", test.name).as_str());

        let mut command = Command::new(bin_path.join(Path::new(&test.binary)));
        command
            .current_dir(&self.piglit_folder)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .args(&test.args)
            .env("MESA_DEBUG", "silent")
            .env("DEQP_RUNNER_THREAD", runner_thread_index()?.to_string())
            .env("PIGLIT_SOURCE_DIR", &self.piglit_folder)
            .envs(self.config.env.iter());

        let command_line = format!("{:?}", command);

        let mut stderr = Vec::new();
        let mut status = None;

        debug!("Begin test {}", test.name);

        let piglit_result = match command
            .spawn()
            .with_context(|| format!("Failed to spawn {}", &test.binary))
        {
            Ok(mut child) => {
                let stdout = child.stdout.take().context("opening stdout")?;

                let mut r = parse_piglit_results_with_timeout(stdout, self.config.timeout);

                // The child should have run to completion based on parse_piglit_results()
                // consuming its output, but if we had a timeout then we want to kill this run.
                let _ = child.kill();

                // Make sure we reap the child process.
                let child_status = child.wait();

                if let Ok(s) = child_status {
                    status = Some(s);

                    // If the process crashed, then report the case crashed
                    // regardless of whether it produced a plausible piglit
                    // result string.
                    match s.code() {
                        Some(0) | Some(1) => {}
                        _ => {
                            if r.status != Some(DeqpStatus::Timeout) {
                                r.status = Some(DeqpStatus::Crash);
                            }
                        }
                    };
                }

                for line in BufReader::new(child.stderr.as_mut().context("opening stderr")?)
                    .lines()
                    .flatten()
                {
                    stderr.push(line);
                }

                r
            }
            Err(e) => PiglitTestResult {
                status: Some(DeqpStatus::Fail),
                duration: std::time::Duration::new(0, 0),
                subtests: Vec::new(),
                stdout: vec![format!("Error spawning piglit command: {:?}", e)],
            },
        };

        let mut results = Vec::new();
        let translated_result = self.translate_result(
            &DeqpTestResult {
                name: test.name.to_owned(),
                status: piglit_result.status.unwrap_or(DeqpStatus::Crash),
                duration: piglit_result.duration,
            },
            caselist_state,
        );

        for subtest in &piglit_result.subtests {
            let subtest_name =
                format!("{}@{}", test.name, piglit_sanitize_test_name(&subtest.name));

            if self.skip_test(&subtest_name) {
                error!(
                    "Skip list matches subtest {}, but you can't skip execution of subtests.",
                    &subtest_name
                );
            }

            results.push(RunnerResult {
                test: subtest_name.clone(),
                status: self.translate_result(
                    &DeqpTestResult {
                        name: subtest_name,
                        status: subtest.status,
                        duration: subtest.duration,
                    },
                    caselist_state,
                ),
                duration: subtest.duration.as_secs_f32(),
                subtest: true,
            });
        }

        if translated_result.should_save_logs(self.config.save_xfail_logs)
            || test.name.contains("glinfo")
        {
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
                writeln!(file, "test: {}", test.name)?;
                writeln!(file, "command: {}", command_line)?;
                if let Some(status) = status {
                    writeln!(file, "exit status: {}", status)?;
                }
                write_output(&mut file, "stdout", &piglit_result.stdout)?;
                write_output(&mut file, "stderr", &stderr)?;
                Ok(())
            }()
            .context("writing log file")?;
        }

        results.push(RunnerResult {
            test: test.name.to_owned(),
            status: translated_result,
            duration: piglit_result.duration.as_secs_f32(),
            subtest: false,
        });

        debug!("End test {}", test.name);

        Ok(results)
    }

    fn see_more(&self, test_name: &str, _caselist_state: &CaselistState) -> String {
        let log_path = self
            .config
            .output_dir
            .join(format!("piglit.{}.log", test_name).as_str());
        format!("See {:?}", log_path)
    }

    fn config(&self) -> &TestConfiguration {
        &self.config
    }
}

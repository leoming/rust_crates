// Copyright (c) 2021 Advanced Micro Devices, Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE
// USE OR OTHER DEALINGS IN THE SOFTWARE.

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use log::*;
use regex::Regex;
use roxmltree::Document;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};
use timeout_readwrite::TimeoutReader;

use crate::parse_deqp::{DeqpStatus, DeqpTestResult};
use crate::TestCase;

impl DeqpStatus {
    // Parses the status name from piglit's output.
    fn from_piglit_str(input: &str) -> Result<DeqpStatus, anyhow::Error> {
        match input {
            "pass" => Ok(DeqpStatus::Pass),
            "fail" => Ok(DeqpStatus::Fail),
            "warn" => Ok(DeqpStatus::CompatibilityWarning),
            "crash" => Ok(DeqpStatus::Crash),
            "skip" => Ok(DeqpStatus::NotSupported),
            "timeout" => Ok(DeqpStatus::Timeout),
            _ => anyhow::bail!("unknown piglit status '{}'", input),
        }
    }
}

#[derive(Debug)]
pub struct PiglitTestResult {
    pub status: Option<DeqpStatus>,
    pub duration: Duration,

    pub subtests: Vec<DeqpTestResult>,
    pub stdout: Vec<String>,
}

// For comparing equality, we ignore the test runtime (particularly of use for the unit tests )
impl PartialEq for PiglitTestResult {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
            && self.subtests == other.subtests
            && self.stdout == other.stdout
    }
}

pub fn parse_piglit_results(piglit_output: impl Read) -> PiglitTestResult {
    let piglit_output = BufReader::new(piglit_output);

    // We only parse the final result, not any subtests, as we don't have a
    // mechanism for reporting the subtests in the main deqp-runner lib.
    lazy_static! {
        static ref STATUS_RE: Regex = Regex::new(r#"PIGLIT: \{"result": "(.*)" \}"#).unwrap();
        static ref SUBTEST_RE: Regex =
            Regex::new(r#"PIGLIT: \{"subtest": *\{"(.*)" *: *"(.*)"\}\}"#).unwrap();
    }

    let mut stdout: Vec<String> = Vec::new();

    // If we don't make it to the final piglit result, assume that the test
    // crashed.
    let mut status = None;

    let startup = Instant::now();

    let mut subtests = Vec::new();

    for line in piglit_output.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                if let std::io::ErrorKind::TimedOut = e.kind() {
                    status = Some(DeqpStatus::Timeout)
                } else {
                    error!("reading from piglit: {:?}", e);
                    status = Some(DeqpStatus::Crash)
                }

                break;
            }
        };

        if let Some(cap) = STATUS_RE.captures(&line) {
            if let Some(old_status) = status {
                error!(
                    "Second piglit status result found (was {:?}, new result {})",
                    old_status, &line
                );
                status = Some(DeqpStatus::Crash);
            } else {
                status = Some(DeqpStatus::from_piglit_str(&cap[1]).unwrap_or_else(|e| {
                    error!("{:?}", e);
                    DeqpStatus::Crash
                }));
            }
        } else if let Some(cap) = SUBTEST_RE.captures(&line) {
            let sub_name = &cap[1];
            let sub_status = DeqpStatus::from_piglit_str(&cap[2]).unwrap_or_else(|e| {
                error!("{:?}", e);
                DeqpStatus::Crash
            });

            if let Some(pos) = subtests
                .iter()
                .position(|x: &DeqpTestResult| x.name == sub_name)
            {
                error!("Duplicate subtest found, marking test failed: {}", sub_name);
                subtests[pos].status = DeqpStatus::Fail;
            } else {
                subtests.push(DeqpTestResult {
                    name: sub_name.to_owned(),
                    status: sub_status,
                    duration: Duration::from_secs_f32(0.0),
                });
            }
        }

        stdout.push(line);
    }

    PiglitTestResult {
        status,
        duration: startup.elapsed(),
        subtests,
        stdout,
    }
}

pub fn parse_piglit_results_with_timeout(
    deqp_output: impl Read + std::os::unix::io::AsRawFd,
    timeout: Duration,
) -> PiglitTestResult {
    parse_piglit_results(TimeoutReader::new(deqp_output, timeout))
}

pub fn read_profile_file(
    piglit_folder: &std::path::Path,
    profile: &str,
    process_isolation: bool,
) -> Result<String> {
    // TODO: Don't read no_isolation version: deqp-runner is the one dispatching
    // multiple tests to a single runner. But this requires parsing the shader_test file
    // to determine which ones can run together (see piglit's shader_test.py)
    if !process_isolation {
        let path = piglit_folder.join(Path::new(profile).with_extension("no_isolation.meta.xml"));
        if path.exists() {
            info!("... using {:?}", &path);
            return std::fs::read_to_string(&path).with_context(|| format!("reading {:?}", path));
        }
    }

    {
        let path = piglit_folder.join(Path::new(profile).with_extension("meta.xml"));
        if path.exists() {
            info!("... using {:?}", path);
            return std::fs::read_to_string(&path).with_context(|| format!("reading {:?}", path));
        }
    }

    /* try .no_isolation.xml.gz first */
    if !process_isolation {
        let path = piglit_folder.join(Path::new(profile).with_extension("no_isolation.xml.gz"));
        if path.exists() {
            info!("... using {:?}", path);
            let file = File::open(&path).with_context(|| format!("opening {:?}", path))?;
            let mut s = String::new();
            GzDecoder::new(file)
                .read_to_string(&mut s)
                .with_context(|| format!("reading {:?}", path))?;
            return Ok(s);
        }
    }

    /* then try .no_isolation.xml */
    if !process_isolation {
        let path = piglit_folder.join(Path::new(profile).with_extension("no_isolation.xml"));
        if path.exists() {
            info!("... using {:?}", path);
            return std::fs::read_to_string(&path).with_context(|| format!("reading {:?}", path));
        }
    }

    /* then try .xml.gz */
    {
        let path = piglit_folder.join(Path::new(profile).with_extension("xml.gz"));
        if path.exists() {
            info!("... using {:?}", path);
            let file = File::open(&path).with_context(|| format!("opening {:?}", path))?;
            let mut s = String::new();
            GzDecoder::new(file)
                .read_to_string(&mut s)
                .with_context(|| format!("reading {:?}", path))?;
            return Ok(s);
        }
    }

    {
        let path = piglit_folder.join(Path::new(profile).with_extension("xml"));
        std::fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))
    }
}

fn get_option_value(node: &roxmltree::Node, name: &str) -> Result<String> {
    let mut children = node.children().filter(|x| x.has_tag_name("option"));
    let option_node = children
        .find(|x| x.attribute("name") == Some(name))
        .with_context(|| format!("Getting option {}", name))?;

    if children.any(|x| x.attribute("name") == Some(name)) {
        anyhow::bail!("More than one option named {}", name);
    }

    Ok(option_node
        .attribute("value")
        .with_context(|| format!("getting option {} value", name))?
        .to_string())
}

fn parse_piglit_command(
    test_name: &str,
    test_type: &str,
    run_concurrent: bool,
    command: &str,
) -> Result<TestCase> {
    let len = command.len();
    if len < 2 {
        anyhow::bail!("command length {} too short", len);
    }
    // The slice strips out the "[]" wrapping the python array dump encoded as
    // an XML string, then we strip off the single quotes around each array element
    // separated by ','.  This still feels fragile.
    let mut all = command[1..(len - 1)]
        .split(',')
        .map(|arg| arg.replace('\'', ""));
    let binary = all.next().context("Getting binary")?;
    let mut args: Vec<String> = all
        .map(|arg| arg.trim().to_string())
        .filter(|arg| !arg.is_empty())
        .map(|arg| {
            if test_type == "gl_builtin" {
                format!("tests/{}", arg)
            } else {
                arg
            }
        })
        .collect();

    if test_type != "glsl_parser" {
        args.push("-auto".to_string());
        if run_concurrent {
            args.push("-fbo".to_string());
        }
    }

    Ok(TestCase::Piglit(crate::PiglitTest {
        name: test_name.to_string(),
        binary,
        args,
    }))
}

/* Replace ',' to avoid conflicts in the csv file */
pub fn piglit_sanitize_test_name(test: &str) -> String {
    test.replace(',', "-")
}

pub fn parse_piglit_xml_testlist(
    folder: &Path,
    file_content: &str,
    process_isolation: bool,
) -> Result<Vec<crate::TestCase>> {
    let doc = Document::parse(file_content).context("reading caselist")?;

    let re = Regex::new(r"['\[\]]").unwrap();
    let mut tests = Vec::new();

    /* meta profile */
    for test in doc.descendants().filter(|n| n.has_tag_name("Profile")) {
        if let Some(name) = test.text() {
            info!("Found subprofile: {:?}", name);
            let content = read_profile_file(folder, name, process_isolation)?;
            for t in parse_piglit_xml_testlist(folder, &content, process_isolation)? {
                tests.push(t);
            }
        }
    }

    for test in doc.descendants().filter(|n| n.has_tag_name("Test")) {
        let test_name =
            piglit_sanitize_test_name(test.attribute("name").context("getting test name")?);

        match test.attribute("type") {
            // multi_shader implements the same feature as deqp-runner: grouping test runs in a
            // single process.
            Some("multi_shader") => {
                let nodes: Vec<roxmltree::Node> = test
                    .children()
                    .filter(|n| n.attribute("name") == Some("files"))
                    .collect();
                if !nodes.is_empty() {
                    let content = re.replace_all(nodes[0].attribute("value").unwrap(), "");
                    let all: Vec<&str> = content.split(',').collect();

                    let mut args = Vec::new();

                    for a in all {
                        let m = a.trim();
                        if !m.is_empty() {
                            args.push(m.to_string());
                        }
                    }

                    let mut remaining = args.len();
                    let mut i = 0u32;
                    while remaining != 0 {
                        let group_len = usize::min(100, remaining);
                        remaining -= group_len;

                        let mut a = args.split_off(remaining);
                        a.push("-auto".to_string());
                        a.push("-fbo".to_string());

                        tests.push(TestCase::Piglit(crate::PiglitTest {
                            name: format!("{}|{}", test_name, i),
                            binary: "shader_runner".to_string(),
                            args: a,
                        }));

                        i += 1;
                    }
                }
            }

            Some("asm_parser") => {
                tests.push(TestCase::Piglit(crate::PiglitTest {
                    name: test_name,
                    binary: "asmparsertest".to_string(),
                    args: vec![
                        get_option_value(&test, "type_")?.replace('\'', ""),
                        get_option_value(&test, "filename")?.replace('\'', ""),
                    ],
                }));
            }

            Some(test_type) => {
                let command = get_option_value(&test, "command").context("parsing command")?;

                let run_concurrent = match get_option_value(&test, "run_concurrent")
                    .context("parsing concurrent")?
                    .as_str()
                {
                    "True" => true,
                    "False" => false,
                    x => anyhow::bail!("Unknown run_concurrent value {}", x),
                };

                tests.push(parse_piglit_command(
                    &test_name,
                    test_type,
                    run_concurrent,
                    &command,
                )?);
            }

            None => anyhow::bail!("No test type specified for {}", test_name),
        }
    }

    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    fn parse_immediate_xml(xml: &str) -> Result<Vec<TestCase>> {
        let dummy_path = PathBuf::from(".");
        parse_piglit_xml_testlist(&dummy_path, xml, false)
    }

    #[test]
    fn parse_command_and_args() {
        let xml = r#"
        <Test type="gl" name="fast_color_clear@fcc-read-after-clear blit rb">
        <option name="command" value="['fcc-read-after-clear', 'blit', 'rb']" />
        <option name="run_concurrent" value="True" />
        </Test>"#;

        assert_eq!(
            parse_immediate_xml(xml).unwrap()[0],
            TestCase::Piglit(crate::PiglitTest {
                name: "fast_color_clear@fcc-read-after-clear blit rb".to_string(),
                binary: "fcc-read-after-clear".to_string(),
                args: vec![
                    "blit".to_string(),
                    "rb".to_string(),
                    "-auto".to_string(),
                    "-fbo".to_string(),
                ],
            })
        );
    }

    #[test]
    fn parse_asmparsertest() {
        let xml = r#"
        <Test type="asm_parser" name="asmparsertest@arbfp1.0@cos-03.txt">
        <option name="type_" value="'ARBfp1.0'" />
        <option name="filename" value="'tests/asmparsertest/shaders/ARBfp1.0/cos-03.txt'" />
        </Test>"#;

        assert_eq!(
            parse_immediate_xml(xml).unwrap()[0],
            TestCase::Piglit(crate::PiglitTest {
                name: "asmparsertest@arbfp1.0@cos-03.txt".to_string(),
                binary: "asmparsertest".to_string(),
                args: vec![
                    "ARBfp1.0".to_string(),
                    "tests/asmparsertest/shaders/ARBfp1.0/cos-03.txt".to_string()
                ],
            })
        );
    }

    #[test]
    fn parse_glslparsertest() {
        let xml = r#"
        <Test type="glsl_parser" name="spec@ext_clip_cull_distance@preprocessor@disabled-defined-es.comp">
        <option name="shader_version" value="3.1" />
        <option name="api" value="'gles3'" />
        <option name="command" value="['glslparsertest_gles2', 'generated_tests/spec/ext_clip_cull_distance/preprocessor/disabled-defined-es.comp', 'pass', '3.10 es', '!GL_EXT_clip_cull_distance']" />
        <option name="run_concurrent" value="True" />
        </Test>"#;

        assert_eq!(
            parse_immediate_xml(xml).unwrap()[0],
            TestCase::Piglit(crate::PiglitTest {
                name: "spec@ext_clip_cull_distance@preprocessor@disabled-defined-es.comp"
                    .to_string(),
                binary: "glslparsertest_gles2".to_string(),
                args: vec![
                    "generated_tests/spec/ext_clip_cull_distance/preprocessor/disabled-defined-es.comp".to_string(),
                    "pass".to_string(), "3.10 es".to_string(), "!GL_EXT_clip_cull_distance".to_string()
                ],
            })
        );
    }

    #[test]
    fn parse_glx_test() {
        // This test must be run without -fbo, or it fails.
        let xml = r#"
        <Test type="gl" name="glx@glx-query-drawable-glxbaddrawable">
        <option name="require_platforms" value="['glx', 'mixed_glx_egl']" />
        <option name="command" value="['glx-query-drawable', '--bad-drawable']" />
        <option name="run_concurrent" value="False" />
        </Test>"#;

        assert_eq!(
            parse_immediate_xml(xml).unwrap()[0],
            TestCase::Piglit(crate::PiglitTest {
                name: "glx@glx-query-drawable-glxbaddrawable".to_string(),
                binary: "glx-query-drawable".to_string(),
                args: vec!["--bad-drawable".to_string(), "-auto".to_string()],
            })
        );
    }

    #[test]
    fn parse_command_with_brackets() -> Result<()> {
        let test = parse_piglit_command(
            "test",
            "gl",
            true,
            r#"['ext_transform_feedback-output-type', 'vec4', '[2]']"#,
        )?;

        assert_eq!(
            test,
            TestCase::Piglit(crate::PiglitTest {
                name: "test".to_string(),
                binary: "ext_transform_feedback-output-type".to_string(),
                args: vec!(
                    "vec4".to_string(),
                    "[2]".to_string(),
                    "-auto".to_string(),
                    "-fbo".to_string(),
                )
            })
        );

        Ok(())
    }

    fn output_as_lines(output: &str) -> Vec<String> {
        output.lines().map(|x| x.to_string()).collect()
    }

    fn result(status: DeqpStatus, orig_output: &str) -> PiglitTestResult {
        PiglitTestResult {
            status: Some(status),
            duration: Duration::new(0, 0),
            subtests: Vec::new(),
            stdout: output_as_lines(orig_output),
        }
    }

    #[test]
    fn parse_statuses() {
        let output = "
PIGLIT: {\"result\": \"pass\" }";

        assert_eq!(
            parse_piglit_results(&mut output.as_bytes()),
            result(DeqpStatus::Pass, output),
        );
    }

    #[test]
    fn parse_subtests() {
        let output = "
PIGLIT: {\"enumerate subtests\": [\"Check valid integer border color values\", \"Check invalid integer border color values\", \"Check valid float border color values\", \"Check invalid float border color values\"]}
PIGLIT: {\"subtest\": {\"Check valid integer border color values\" : \"pass\"}}
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
PIGLIT: {\"subtest\": {\"Check invalid integer border color values\" : \"fail\"}}
PIGLIT: {\"subtest\": {\"Check valid float border color values\" : \"skip\"}}
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
Mesa: User error: GL_INVALID_OPERATION in glGetTextureSamplerHandleARB(invalid border color)
PIGLIT: {\"subtest\": {\"Check invalid float border color values\" : \"warn\"}}
PIGLIT: {\"result\": \"pass\" }";

        let mut expected_result = result(DeqpStatus::Pass, output);
        expected_result.subtests.push(DeqpTestResult {
            name: "Check valid integer border color values".to_owned(),
            status: DeqpStatus::Pass,
            duration: Duration::new(0, 0),
        });
        expected_result.subtests.push(DeqpTestResult {
            name: "Check invalid integer border color values".to_owned(),
            status: DeqpStatus::Fail,
            duration: Duration::new(0, 0),
        });
        expected_result.subtests.push(DeqpTestResult {
            name: "Check valid float border color values".to_owned(),
            status: DeqpStatus::NotSupported,
            duration: Duration::new(0, 0),
        });
        expected_result.subtests.push(DeqpTestResult {
            name: "Check invalid float border color values".to_owned(),
            status: DeqpStatus::CompatibilityWarning,
            duration: Duration::new(0, 0),
        });

        assert_eq!(parse_piglit_results(output.as_bytes()), expected_result);
    }

    fn timeout_test(output: &str) -> Result<PiglitTestResult> {
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = std::io::BufWriter::new(child.stdin.take().unwrap());

        stdin.write_all(output.as_bytes()).unwrap();
        stdin.flush().unwrap();

        let stdout = child.stdout.take().context("opening stdout")?;

        let result = parse_piglit_results_with_timeout(stdout, Duration::new(0, 100_000_000));

        child.kill().context("killing cat")?;

        Ok(result)
    }

    #[test]
    fn parse_crash() -> Result<()> {
        let output = r#"
        PIGLIT: {"subtest": {"Vertex shader/control memory barrier test/modulus=1" : "pass"}}
        PIGLIT: {"subtest": {"Tessellation control shader/control memory barrier test/modulus=1" : "pass"}}"#;

        assert_eq!(
            parse_piglit_results(output.as_bytes()),
            PiglitTestResult {
                status: None,
                stdout: output_as_lines(output),
                subtests: vec![
                    DeqpTestResult {
                        name: "Vertex shader/control memory barrier test/modulus=1".to_owned(),
                        status: DeqpStatus::Pass,
                        duration: Duration::new(0, 0),
                    },
                    DeqpTestResult {
                        name: "Tessellation control shader/control memory barrier test/modulus=1"
                            .to_owned(),
                        status: DeqpStatus::Pass,
                        duration: Duration::new(0, 0),
                    }
                ],
                duration: Duration::new(0, 0),
            }
        );

        Ok(())
    }

    #[test]
    fn parse_timeout_no_result() -> Result<()> {
        let output = "Starting piglit test...\n";
        assert_eq!(timeout_test(output)?, result(DeqpStatus::Timeout, output),);
        Ok(())
    }

    #[test]
    fn parse_timeout_result() -> Result<()> {
        let output = r#"PIGLIT: {"result": "pass" }
"#;

        assert_eq!(timeout_test(output)?, result(DeqpStatus::Timeout, output),);
        Ok(())
    }

    #[test]
    fn parse_shader_runner_subtests() -> Result<()> {
        let output = r#"
PIGLIT TEST: 1 - glsl-fs-swizzle-1
PIGLIT TEST: 1 - glsl-fs-swizzle-1
PIGLIT: {"subtest": {"glsl-fs-swizzle-1" : "pass"}}
PIGLIT TEST: 2 - vs-sign-neg
PIGLIT TEST: 2 - vs-sign-neg
PIGLIT: {"subtest": {"vs-sign-neg" : "pass"}}
"#;

        let results = parse_piglit_results(output.as_bytes());

        assert_eq!(results.status, None);
        assert_eq!(
            results.subtests,
            vec![
                DeqpTestResult {
                    name: "glsl-fs-swizzle-1".to_owned(),
                    status: DeqpStatus::Pass,
                    duration: Duration::new(0, 0)
                },
                DeqpTestResult {
                    name: "vs-sign-neg".to_owned(),
                    status: DeqpStatus::Pass,
                    duration: Duration::new(0, 0)
                },
            ]
        );
        Ok(())
    }
}

use crate::config::abbrev::Operation;
use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult<'a> {
    pub lbuffer: &'a str,
    pub startindex: usize,
    pub endindex: usize,
    pub last_arg: &'a str,
    pub snippet: &'a str,
    pub evaluate: bool,
    pub rbuffer: &'a str,
}

pub fn run(args: &ExpandArgs) {
    if let Some(result) = expand(args, &Config::load_or_exit()) {
        let lbuffer_prev = escape(Cow::from(&result.lbuffer[..result.startindex]));
        let lbuffer_post = escape(Cow::from(&result.lbuffer[result.endindex..]));
        let snippet = escape(Cow::from(result.snippet));
        let rbuffer = escape(Cow::from(result.rbuffer));

        let (joint_append, joint_prepend) = if result.startindex == result.endindex {
            if result.startindex == result.lbuffer.len() {
                (" ", "")
            } else {
                ("", " ")
            }
        } else {
            ("", "")
        };

        if result.evaluate {
            println!(
                r#"local prev={};local post={};local snippet={};LBUFFER="${{prev}}{}${{snippet}}{}${{post}}";RBUFFER={};"#,
                lbuffer_prev, lbuffer_post, snippet, joint_append, joint_prepend, rbuffer
            );
        } else {
            let last_arg = escape(Cow::from(result.last_arg));
            println!(
                r#"local prev={};local post={};local snippet={} {};LBUFFER="${{prev}}{}${{(e)snippet}}{}${{post}}";RBUFFER={};"#,
                lbuffer_prev, lbuffer_post, snippet, last_arg, joint_append, joint_prepend, rbuffer
            );
        }
    }
}

fn expand<'a>(args: &'a ExpandArgs, config: &'a Config) -> Option<ExpandResult<'a>> {
    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let command_index = find_last_command_index(lbuffer);
    let command = lbuffer[command_index..].trim_start();

    let (args_until_last, last_arg) = command
        .rsplit_once(char::is_whitespace)
        .unwrap_or(("", command));

    if last_arg.is_empty() {
        return None;
    }

    let (context, internal_args) = args_until_last
        .split_once(char::is_whitespace)
        .unwrap_or((args_until_last, ""));

    let abbrev = config
        .abbrevs
        .iter()
        .find(|abbr| abbr.is_match(command, context, last_arg, internal_args.is_empty()))?;

    let (startindex, endindex) = match abbrev.operation {
        Operation::ReplaceSelf => {
            let index = lbuffer.len() - last_arg.len();
            (index, lbuffer.len())
        }
        Operation::ReplaceCommand => {
            let index = lbuffer.len() - command.len();
            (index, index + context.len())
        }
        Operation::ReplaceAll => {
            let index = lbuffer.len() - command.len();
            (index, lbuffer.len())
        }
        Operation::Append => {
            let index = lbuffer.len();
            (index, index)
        }
        Operation::Prepend => {
            let index = lbuffer.len() - command.len();
            (index, index)
        }
    };

    Some(ExpandResult {
        lbuffer,
        startindex,
        endindex,
        last_arg,
        snippet: &abbrev.snippet,
        evaluate: abbrev.evaluate,
        rbuffer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config::load_from_str(
            r"
            abbrevs:
              - name: git
                abbr: g
                snippet: git

              - name: git commit
                abbr: c
                snippet: commit
                global: false
                context: 'git'

              - name: '>/dev/null'
                abbr: 'null'
                snippet: '>/dev/null'
                global: true

              - name: $HOME
                abbr: home
                snippet: $HOME
                evaluate: true

              - name: default argument
                abbr: rm
                snippet: -i
                operation: append

              - name: fake command
                context: 'extract'
                regex: '\.tar$'
                snippet: 'tar -xvf'
                operation: replace-command

              - name: associated command
                regex: '\.java$'
                snippet: 'java -jar'
                operation: prepend
            ",
        )
        .unwrap()
    }

    #[test]
    fn test_expand() {
        let config = test_config();

        struct Scenario<'a> {
            pub testname: &'a str,
            pub lbuffer: &'a str,
            pub rbuffer: &'a str,
            pub expected: Option<ExpandResult<'a>>,
        }

        let scenarios = &[
            Scenario {
                testname: "empty",
                lbuffer: "",
                rbuffer: "",
                expected: None,
            },
            Scenario {
                testname: "simple abbr",
                lbuffer: "g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "g",
                    startindex: 0,
                    endindex: 1,
                    last_arg: "g",
                    snippet: "git",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "simple abbr with rbuffer",
                lbuffer: "g",
                rbuffer: " --pager=never",
                expected: Some(ExpandResult {
                    lbuffer: "g",
                    startindex: 0,
                    endindex: 1,
                    last_arg: "g",
                    snippet: "git",
                    evaluate: false,
                    rbuffer: " --pager=never",
                }),
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; g",
                    startindex: 12,
                    endindex: 13,
                    last_arg: "g",
                    snippet: "git",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello null",
                    startindex: 11,
                    endindex: 15,
                    last_arg: "null",
                    snippet: ">/dev/null",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                rbuffer: " -m hello",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; git c",
                    startindex: 16,
                    endindex: 17,
                    last_arg: "c",
                    snippet: "commit",
                    evaluate: false,
                    rbuffer: " -m hello",
                }),
            },
            Scenario {
                testname: "global abbr with miss matched context",
                lbuffer: "echo git c",
                rbuffer: "",
                expected: None,
            },
            Scenario {
                testname: "no matched abbr",
                lbuffer: "echo",
                rbuffer: " hello",
                expected: None,
            },
            Scenario {
                testname: "simple abbr with evaluate=true",
                lbuffer: "home",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "home",
                    startindex: 0,
                    endindex: 4,
                    last_arg: "home",
                    snippet: "$HOME",
                    evaluate: true,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "default argument abbr",
                lbuffer: "rm",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "rm",
                    startindex: 2,
                    endindex: 2,
                    last_arg: "rm",
                    snippet: "-i",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "fake command abbr",
                lbuffer: "extract test.tar",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "extract test.tar",
                    startindex: 0,
                    endindex: 7,
                    last_arg: "test.tar",
                    snippet: "tar -xvf",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "associated command abbr",
                lbuffer: "test.java",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "test.java",
                    startindex: 0,
                    endindex: 0,
                    last_arg: "test.java",
                    snippet: "java -jar",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
        ];

        for s in scenarios {
            let args = ExpandArgs {
                lbuffer: s.lbuffer.to_string(),
                rbuffer: s.rbuffer.to_string(),
            };

            let actual = expand(&args, &config);

            assert_eq!(actual, s.expected, "{}", s.testname);
        }
    }
}

fn find_last_command_index(line: &str) -> usize {
    line.rfind(|c| matches!(c, ';' | '&' | '|' | '(' | '`' | '\n'))
        .map(|i| i + 1)
        .unwrap_or(0)
}

#[test]
fn test_find_last_command_index() {
    assert_eq!(find_last_command_index("git commit"), 0);
    assert_eq!(find_last_command_index("echo hello; git commit"), 11);
    assert_eq!(find_last_command_index("echo hello && git commit"), 13);
    assert_eq!(find_last_command_index("seq 10 | tail -3 | cat"), 18);
}

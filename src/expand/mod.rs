use crate::config::abbrev::Operation;
use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult<'a> {
    pub lbuffer: &'a str,
    pub rbuffer: &'a str,
    pub last_arg: &'a str,
    pub snippet: &'a str,
    pub start_index_of_replacement: usize,
    pub end_index_of_replacement: usize,
    pub is_append: bool,
    pub is_prepend: bool,
    pub evaluate: bool,
}

#[derive(Debug, PartialEq)]
pub struct SplitResult<'a> {
    pub args_until_last: Vec<&'a str>,
    pub last_arg: &'a str,
}

pub fn run(args: &ExpandArgs) {
    if let Some(result) = expand(args, &Config::load_or_exit()) {
        let lbuffer_prev = escape(Cow::from(
            &result.lbuffer[..result.start_index_of_replacement],
        ));
        let lbuffer_post = escape(Cow::from(
            &result.lbuffer[result.end_index_of_replacement..],
        ));
        let last_arg = escape(Cow::from(result.last_arg));
        let snippet = escape(Cow::from(result.snippet));
        let rbuffer = escape(Cow::from(result.rbuffer));
        let evaluate = if result.evaluate { "(e)" } else { "" };

        println!(
            r#"local snippet={};set -- {};snippet="${{{}snippet}}";[[ $? -eq 0 ]] && {{ LBUFFER={}"{}${{(pj: :)${{(@f)snippet}}}}{}"{};RBUFFER={};}};"#,
            snippet,
            last_arg,
            evaluate,
            lbuffer_prev,
            if result.is_append { " " } else { "" },
            if result.is_prepend { " " } else { "" },
            lbuffer_post,
            rbuffer
        );
    }
}

fn expand<'a>(args: &'a ExpandArgs, config: &'a Config) -> Option<ExpandResult<'a>> {
    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let command_index = find_last_command_index(lbuffer);
    let command = lbuffer[command_index..].trim_start();

    let SplitResult {
        args_until_last,
        last_arg,
    } = split_args(command);

    if last_arg.is_empty() {
        return None;
    }

    let match_result = config
        .abbrevs
        .iter()
        .flat_map(|abbr| abbr.matches(&args_until_last, last_arg))
        .next()?;

    let (start_index_of_replacement, end_index_of_replacement, is_append, is_prepend) =
        match match_result.abbrev.operation {
            Operation::ReplaceSelf => {
                let index = lbuffer.len() - last_arg.len();
                (index, lbuffer.len(), false, false)
            }
            Operation::ReplaceFirst => {
                let index = lbuffer.len() - command.len();
                let len = args_until_last
                    .first()
                    .map(|&x| x.len())
                    .unwrap_or(last_arg.len());
                (index, index + len, false, false)
            }
            Operation::ReplaceContext => {
                let index = lbuffer.len() - command.len();
                match match_result.context_size {
                    0 => (index, index, false, true),
                    context_size => {
                        let last_arg_of_context = args_until_last[context_size - 1];
                        (
                            index,
                            unsafe { get_subslice_index_unchecked(lbuffer, last_arg_of_context) }
                                + last_arg_of_context.len(),
                            false,
                            false,
                        )
                    }
                }
            }
            Operation::ReplaceAll => {
                let index = lbuffer.len() - command.len();
                (index, lbuffer.len(), false, false)
            }
            Operation::Append => {
                let index = lbuffer.len();
                (index, index, true, false)
            }
            Operation::Prepend => {
                let index = lbuffer.len() - command.len();
                (index, index, false, true)
            }
        };

    Some(ExpandResult {
        lbuffer,
        rbuffer,
        start_index_of_replacement,
        end_index_of_replacement,
        last_arg,
        snippet: &match_result.abbrev.snippet,
        is_append,
        is_prepend,
        evaluate: match_result.abbrev.evaluate,
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
                abbr-regex: '\.tar$'
                snippet: 'tar -xvf'
                operation: replace-first

              - name: 'function?'
                context: 'mkdircd'
                abbr-regex: '.+'
                snippet: 'mkdir -p $1 && cd $1'
                operation: replace-all
                evaluate: true

              - name: associated command
                abbr-regex: '\.java$'
                snippet: 'java -jar'
                operation: prepend

              - name: context replacement
                context: 'a b'
                abbr: c
                snippet: 'A'
                operation: replace-context
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
                    rbuffer: "",
                    start_index_of_replacement: 0,
                    end_index_of_replacement: 1,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "g",
                    snippet: "git",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "simple abbr with rbuffer",
                lbuffer: "g",
                rbuffer: " --pager=never",
                expected: Some(ExpandResult {
                    lbuffer: "g",
                    rbuffer: " --pager=never",
                    start_index_of_replacement: 0,
                    end_index_of_replacement: 1,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "g",
                    snippet: "git",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; g",
                    rbuffer: "",
                    start_index_of_replacement: 12,
                    end_index_of_replacement: 13,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "g",
                    snippet: "git",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello null",
                    rbuffer: "",
                    start_index_of_replacement: 11,
                    end_index_of_replacement: 15,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "null",
                    snippet: ">/dev/null",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                rbuffer: " -m hello",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; git c",
                    rbuffer: " -m hello",
                    start_index_of_replacement: 16,
                    end_index_of_replacement: 17,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "c",
                    snippet: "commit",
                    evaluate: false,
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
                    rbuffer: "",
                    start_index_of_replacement: 0,
                    end_index_of_replacement: 4,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "home",
                    snippet: "$HOME",
                    evaluate: true,
                }),
            },
            Scenario {
                testname: "default argument abbr",
                lbuffer: "rm",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "rm",
                    rbuffer: "",
                    start_index_of_replacement: 2,
                    end_index_of_replacement: 2,
                    is_append: true,
                    is_prepend: false,
                    last_arg: "rm",
                    snippet: "-i",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "fake command abbr",
                lbuffer: "extract test.tar",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "extract test.tar",
                    rbuffer: "",
                    start_index_of_replacement: 0,
                    end_index_of_replacement: 7,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "test.tar",
                    snippet: "tar -xvf",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "like a function abbr",
                lbuffer: "mkdircd foo/bar",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "mkdircd foo/bar",
                    rbuffer: "",
                    start_index_of_replacement: 0,
                    end_index_of_replacement: 15,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "foo/bar",
                    snippet: "mkdir -p $1 && cd $1",
                    evaluate: true,
                }),
            },
            Scenario {
                testname: "associated command abbr",
                lbuffer: "test.java",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "test.java",
                    rbuffer: "",
                    start_index_of_replacement: 0,
                    end_index_of_replacement: 0,
                    is_append: false,
                    is_prepend: true,
                    last_arg: "test.java",
                    snippet: "java -jar",
                    evaluate: false,
                }),
            },
            Scenario {
                testname: "context replacement",
                lbuffer: " a b c",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: " a b c",
                    rbuffer: "",
                    start_index_of_replacement: 1,
                    end_index_of_replacement: 4,
                    is_append: false,
                    is_prepend: false,
                    last_arg: "c",
                    snippet: "A",
                    evaluate: false,
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

enum SplitState {
    Delimiter,
    Backslash,
    InWord { is_escaped: bool },
    InQuot { quot: char, is_escaped: bool },
}
impl Default for SplitState {
    fn default() -> Self {
        SplitState::Delimiter
    }
}

fn split_args<'a>(command: &'a str) -> SplitResult {
    use SplitState::*;

    let mut start = 0;
    let mut args_until_last = Vec::new();
    let mut state = SplitState::default();
    let mut ite = command.char_indices();

    loop {
        match ite.next() {
            Some((idx, c)) => {
                state = match state {
                    Delimiter => match c {
                        '\\' => {
                            start = idx;
                            Backslash
                        }
                        '\'' | '\"' => {
                            start = idx;
                            InQuot {
                                quot: c,
                                is_escaped: false,
                            }
                        }
                        ' ' | '\t' | '\n' => Delimiter,
                        _ => {
                            start = idx;
                            InWord { is_escaped: false }
                        }
                    },
                    InWord { is_escaped: false } => match c {
                        '\\' => InWord { is_escaped: true },
                        '\'' | '\"' => InQuot {
                            quot: c,
                            is_escaped: false,
                        },
                        ' ' | '\t' | '\n' => {
                            args_until_last.push(&command[start..idx]);
                            Delimiter
                        }
                        _ => InWord { is_escaped: false },
                    },
                    InQuot {
                        quot,
                        is_escaped: false,
                    } => match c {
                        _ if c == quot => InWord { is_escaped: false },
                        '\\' => InQuot {
                            quot,
                            is_escaped: true,
                        },
                        _ => InQuot {
                            quot,
                            is_escaped: false,
                        },
                    },
                    Backslash => match c {
                        '\n' => Delimiter,
                        _ => InWord { is_escaped: false },
                    },
                    InWord { is_escaped: true } => InWord { is_escaped: false },
                    InQuot {
                        quot,
                        is_escaped: true,
                    } => InQuot {
                        quot,
                        is_escaped: false,
                    },
                }
            }
            None => {
                let last_arg = match state {
                    Delimiter => &command[command.len()..],
                    _ => &command[start..],
                };
                return SplitResult {
                    args_until_last,
                    last_arg,
                };
            }
        }
    }
}

#[test]
fn test_split_args() {
    assert_eq!(
        split_args(""),
        SplitResult {
            args_until_last: vec![],
            last_arg: "",
        }
    );
    assert_eq!(
        split_args(" "),
        SplitResult {
            args_until_last: vec![],
            last_arg: "",
        }
    );
    assert_eq!(
        split_args(":"),
        SplitResult {
            args_until_last: vec![],
            last_arg: ":",
        }
    );
    assert_eq!(
        split_args("\\"),
        SplitResult {
            args_until_last: vec![],
            last_arg: "\\",
        }
    );
    assert_eq!(
        split_args("\'"),
        SplitResult {
            last_arg: "\'",
            args_until_last: vec![],
        }
    );
    assert_eq!(
        split_args("\""),
        SplitResult {
            last_arg: "\"",
            args_until_last: vec![],
        }
    );
    assert_eq!(
        split_args(": "),
        SplitResult {
            args_until_last: vec![":"],
            last_arg: "",
        }
    );
    assert_eq!(
        split_args("\\ "),
        SplitResult {
            args_until_last: vec![],
            last_arg: "\\ ",
        }
    );
    assert_eq!(
        split_args("\' "),
        SplitResult {
            args_until_last: vec![],
            last_arg: "\' ",
        }
    );
    assert_eq!(
        split_args("\" "),
        SplitResult {
            args_until_last: vec![],
            last_arg: "\" ",
        }
    );
    assert_eq!(
        split_args("git"),
        SplitResult {
            args_until_last: vec![],
            last_arg: "git",
        }
    );
    assert_eq!(
        split_args("git commit"),
        SplitResult {
            args_until_last: vec!["git"],
            last_arg: "commit",
        }
    );
    assert_eq!(
        split_args("git  commit"),
        SplitResult {
            args_until_last: vec!["git"],
            last_arg: "commit",
        }
    );
    assert_eq!(
        split_args(" git  commit"),
        SplitResult {
            args_until_last: vec!["git"],
            last_arg: "commit",
        }
    );
    assert_eq!(
        split_args(" git  commit "),
        SplitResult {
            args_until_last: vec!["git", "commit"],
            last_arg: "",
        }
    );
    assert_eq!(
        split_args("git\\ commit"),
        SplitResult {
            args_until_last: vec![],
            last_arg: "git\\ commit",
        }
    );
    assert_eq!(
        split_args("git 'a file.txt'"),
        SplitResult {
            args_until_last: vec!["git"],
            last_arg: "'a file.txt'",
        }
    );
    assert_eq!(
        split_args("git ''a file.txt'"),
        SplitResult {
            args_until_last: vec!["git", "''a"],
            last_arg: "file.txt'",
        }
    );
    assert_eq!(
        split_args("git '''a file.txt'"),
        SplitResult {
            args_until_last: vec!["git"],
            last_arg: "'''a file.txt'",
        }
    );
    assert_eq!(
        split_args("git 'a \\' file.txt'"),
        SplitResult {
            args_until_last: vec!["git"],
            last_arg: "'a \\' file.txt'",
        }
    );
    assert_eq!(
        split_args("git 'a \\\\' file.txt'\\"),
        SplitResult {
            args_until_last: vec!["git", "'a \\\\'"],
            last_arg: "file.txt'\\",
        }
    );
}

unsafe fn get_subslice_index_unchecked<'a>(slice: &'a str, subslice: &'a str) -> usize {
    use std::convert::TryInto;
    subslice
        .as_ptr()
        .offset_from(slice.as_ptr())
        .try_into()
        .unwrap()
}

#[test]
fn test_get_subslice_index_unchecked() {
    let s = "abcdefg";
    for i in 0..s.len() {
        let s2 = &s[i..];
        assert_eq!(unsafe { get_subslice_index_unchecked(s, s2) }, i);
    }
}

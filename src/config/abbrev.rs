use ansi_term::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Trigger {
    #[serde(rename = "abbr")]
    Abbr(String),
    #[serde(rename = "regex")]
    Regex(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    #[serde(rename = "replace-self")]
    ReplaceSelf,
    #[serde(rename = "replace-command")]
    ReplaceCommand,
    #[serde(rename = "replace-all")]
    ReplaceAll,
    #[serde(rename = "append")]
    Append,
    #[serde(rename = "prepend")]
    Prepend,
}
impl Default for Operation {
    fn default() -> Self {
        Operation::ReplaceSelf
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Abbrev {
    pub name: Option<String>,

    #[serde(default)]
    pub context: String,

    #[serde(default = "default_as_false")]
    pub global: bool,

    #[serde(flatten)]
    pub trigger: Trigger,

    pub snippet: String,

    #[serde(default)]
    pub operation: Operation,

    #[serde(default = "default_as_false")]
    pub evaluate: bool,
}

impl Abbrev {
    pub fn is_match(
        &self,
        _command: &str,
        context: &str,
        last_arg: &str,
        is_no_internal_args: bool,
    ) -> bool {
        if !(self.context == "" && self.global) {
            if self.context != context {
                return false;
            }
            if !self.global && !is_no_internal_args {
                return false;
            }
        }

        match self.trigger {
            Trigger::Abbr(ref abbr) => last_arg == abbr,
            Trigger::Regex(ref regex) => {
                let pattern_or_error = Regex::new(regex);
                match pattern_or_error {
                    Ok(pattern) => pattern.is_match(last_arg),
                    Err(err) => {
                        let name = self.name.as_ref().unwrap_or(&self.snippet);
                        let error_message = format!("invalid regex in abbrev '{}': {}", name, err);
                        let error_style = Color::Red.normal();

                        eprintln!("{}", error_style.paint(error_message));
                        false
                    }
                }
            }
        }

        //         let pattern_or_error = match self.context.as_ref().map(|ctx| Regex::new(ctx)) {
        //             Some(pattern_or_error) => pattern_or_error,
        //             None => return true,
        //         };

        //         match pattern_or_error {
        //             Ok(pattern) => pattern.is_match(command),
        //             Err(err) => {
        //                 let name = self.name.as_ref().unwrap_or(&self.snippet);
        //                 let error_message = format!("invalid regex in abbrev '{}': {}", name, err);
        //                 let error_style = Color::Red.normal();

        //                 eprintln!("{}", error_style.paint(error_message));
        //                 false
        //             }
        //         }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match() {
        struct Scenario {
            pub testname: &'static str,
            pub abbr: Abbrev,
            pub command: &'static str,
            pub expected: bool,
        }

        let scenarios = &[
            Scenario {
                testname: "should match non-global if first arg",
                abbr: Abbrev {
                    name: None,
                    context: String::new(),
                    global: false,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "test",
                expected: true,
            },
            Scenario {
                testname: "should not match non-global if second arg",
                abbr: Abbrev {
                    name: None,
                    context: String::new(),
                    global: false,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: false,
            },
            Scenario {
                testname: "should match global if first arg",
                abbr: Abbrev {
                    name: None,
                    context: String::new(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "test",
                expected: true,
            },
            Scenario {
                testname: "should match global if second arg",
                abbr: Abbrev {
                    name: None,
                    context: String::new(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: true,
            },
            Scenario {
                testname: "should match global if third arg",
                abbr: Abbrev {
                    name: None,
                    context: String::new(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo a test",
                expected: true,
            },
            Scenario {
                testname: "should not match non-global with context if first arg",
                abbr: Abbrev {
                    name: None,
                    context: "test".to_string(),
                    global: false,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "test",
                expected: false,
            },
            Scenario {
                testname: "should match non-global with context if second arg",
                abbr: Abbrev {
                    name: None,
                    context: "echo".to_string(),
                    global: false,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: true,
            },
            Scenario {
                testname: "should not match non-global with context if third arg",
                abbr: Abbrev {
                    name: None,
                    context: "echo".to_string(),
                    global: false,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo a test",
                expected: false,
            },
            Scenario {
                testname: "should not match non-global with context mismatch",
                abbr: Abbrev {
                    name: None,
                    context: "printf".to_string(),
                    global: false,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: false,
            },
            Scenario {
                testname: "should not match if context is invalid",
                abbr: Abbrev {
                    name: None,
                    context: "(echo".to_string(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: false,
            },
            Scenario {
                testname: "should not match global with context if first arg",
                abbr: Abbrev {
                    name: None,
                    context: "test".to_string(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "test",
                expected: false,
            },
            Scenario {
                testname: "should match global with context if second arg",
                abbr: Abbrev {
                    name: None,
                    context: "echo".to_string(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: true,
            },
            Scenario {
                testname: "should match global with context if third arg",
                abbr: Abbrev {
                    name: None,
                    context: "echo".to_string(),
                    global: true,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo a test",
                expected: true,
            },
            Scenario {
                testname: "should match regex pattern if first arg",
                abbr: Abbrev {
                    name: None,
                    context: "".to_string(),
                    global: false,
                    trigger: Trigger::Regex(".+".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "test",
                expected: true,
            },
            Scenario {
                testname: "should not match regex pattern if second arg",
                abbr: Abbrev {
                    name: None,
                    context: "".to_string(),
                    global: false,
                    trigger: Trigger::Regex(".+".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: false,
            },
            Scenario {
                testname: "should not match regex pattern if third arg",
                abbr: Abbrev {
                    name: None,
                    context: "".to_string(),
                    global: false,
                    trigger: Trigger::Regex(".+".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo a test",
                expected: false,
            },
            Scenario {
                testname: "should not match regex pattern with context if first arg",
                abbr: Abbrev {
                    name: None,
                    context: "test".to_string(),
                    global: false,
                    trigger: Trigger::Regex(".+".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "test",
                expected: false,
            },
            Scenario {
                testname: "should match regex pattern with context if second arg",
                abbr: Abbrev {
                    name: None,
                    context: "echo".to_string(),
                    global: false,
                    trigger: Trigger::Regex(".+".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo test",
                expected: true,
            },
            Scenario {
                testname: "should not match regex pattern with context if third arg",
                abbr: Abbrev {
                    name: None,
                    context: "echo".to_string(),
                    global: false,
                    trigger: Trigger::Regex(".+".to_string()),
                    snippet: String::new(),
                    operation: Operation::ReplaceSelf,
                    evaluate: false,
                },
                command: "echo a test",
                expected: false,
            },
        ];

        for s in scenarios {
            let (until_last_args, last_arg) = s
                .command
                .rsplit_once(char::is_whitespace)
                .unwrap_or(("", s.command));

            let (context, internal_args) = until_last_args
                .split_once(char::is_whitespace)
                .unwrap_or((until_last_args, ""));

            println!("command = {}", s.command);
            println!("context = {}", context);
            println!("internal_args = {}", internal_args);
            println!("last_arg = {}", last_arg);

            assert_eq!(
                s.abbr
                    .is_match(s.command, context, last_arg, internal_args.is_empty()),
                s.expected,
                "{}",
                s.testname
            );
        }
    }
}

fn default_as_false() -> bool {
    false
}

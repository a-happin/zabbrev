use ansi_term::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    #[serde(default)]
    pub context: String,

    #[serde(default = "default_as_false")]
    pub global: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Trigger {
    #[serde(rename = "abbr")]
    AbbrString(String),
    #[serde(rename = "abbr-regex")]
    AbbrRegex(String),
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

    #[serde(flatten)]
    pub context: Context,

    #[serde(flatten)]
    pub trigger: Trigger,

    pub snippet: String,

    #[serde(default)]
    pub operation: Operation,

    #[serde(default = "default_as_false")]
    pub evaluate: bool,
}

impl Context {
    pub fn is_match(&self, context: &str, args_count_until_last: usize) -> bool {
        if self.global {
            if self.context == "" {
                true
            } else {
                self.context == context
            }
        } else {
            if self.context == "" {
                args_count_until_last == 0
            } else {
                args_count_until_last == 1 && self.context == context
            }
        }
    }
}

impl Trigger {
    pub fn is_match(&self, last_arg: &str) -> Result<bool, regex::Error> {
        match self {
            Self::AbbrString(ref abbr) => Ok(last_arg == abbr),
            Self::AbbrRegex(ref regex) => {
                let pattern = Regex::new(regex)?;
                Ok(pattern.is_match(last_arg))
            }
        }
    }
}

impl Abbrev {
    pub fn is_match(&self, context: &str, args_count_until_last: usize, last_arg: &str) -> bool {
        self.context.is_match(context, args_count_until_last)
            && self.trigger.is_match(last_arg).unwrap_or_else(|err| {
                let name = self.name.as_ref().unwrap_or(&self.snippet);
                let error_message = format!("invalid regex in abbrev '{}': {}", name, err);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                false
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match_context() {
        struct Scenario {
            pub testname: &'static str,
            pub abbr_context: Context,
            pub context: &'static str,
            pub args_count_until_last: usize,
            pub expected: bool,
        }
        let scenarios = &[
            Scenario {
                testname: "should match empty context, non-global, no args_until_last",
                abbr_context: Context {
                    context: String::new(),
                    global: false,
                },
                context: "",
                args_count_until_last: 0,
                expected: true,
            },
            Scenario {
                testname: "should not match empty context, non-global, one or more args_until_last",
                abbr_context: Context {
                    context: String::new(),
                    global: false,
                },
                context: "a",
                args_count_until_last: 1,
                expected: false,
            },
            Scenario {
                testname: "should match empty context, global, no args_until_last",
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                context: "",
                args_count_until_last: 0,
                expected: true,
            },
            Scenario {
                testname: "should match empty context, global, one or more args_until_last",
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                context: "a",
                args_count_until_last: 1,
                expected: true,
            },
            Scenario {
                testname: "should not match with context, non-global, no args_until_last",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                context: "",
                args_count_until_last: 0,
                expected: false,
            },
            Scenario {
                testname:
                    "should not match with context, non-global, args_until_last with wrong context",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                context: "a",
                args_count_until_last: 1,
                expected: false,
            },
            Scenario {
                testname: "should match with context, non-global",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                context: "git",
                args_count_until_last: 1,
                expected: true,
            },
            Scenario {
                testname:
                    "should not match with context, non-global, more than one args_until_last",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                context: "git",
                args_count_until_last: 2,
                expected: false,
            },
            Scenario {
                testname: "should not match with context, global, no args_until_last",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                context: "",
                args_count_until_last: 0,
                expected: false,
            },
            Scenario {
                testname:
                    "should not match with context, global, args_until_last with wrong context",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                context: "a",
                args_count_until_last: 1,
                expected: false,
            },
            Scenario {
                testname: "should match with context, global",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                context: "git",
                args_count_until_last: 1,
                expected: true,
            },
            Scenario {
                testname: "should match with context, global, more than one args_until_last",
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                context: "git",
                args_count_until_last: 2,
                expected: true,
            },
        ];
        for s in scenarios {
            assert_eq!(
                s.abbr_context.is_match(&s.context, s.args_count_until_last),
                s.expected,
                "{}",
                s.testname
            );
        }
    }

    #[test]
    fn test_is_match_trigger() {
        struct Scenario {
            pub testname: &'static str,
            pub trigger: Trigger,
            pub last_arg: &'static str,
            pub expected: Result<bool, regex::Error>,
        }

        let scenarios = &[
            Scenario {
                testname: "should match",
                trigger: Trigger::AbbrString("test".to_string()),
                last_arg: "test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should not match",
                trigger: Trigger::AbbrString("test".to_string()),
                last_arg: "tesr",
                expected: Ok(false),
            },
            Scenario {
                testname: "should match regex",
                trigger: Trigger::AbbrRegex(".+".to_string()),
                last_arg: "test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should match regex",
                trigger: Trigger::AbbrRegex("\\.test$".to_string()),
                last_arg: "atest",
                expected: Ok(false),
            },
        ];
        for s in scenarios {
            assert_eq!(
                s.trigger.is_match(&s.last_arg),
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

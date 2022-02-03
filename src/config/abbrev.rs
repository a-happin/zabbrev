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
    pub fn matches(&self, args_until_last: &[&str]) -> bool {
        let mut context = self.context.trim_start();
        if context.is_empty() {
            self.global || args_until_last.len() == 0
        } else {
            let mut ite = args_until_last.iter();
            loop {
                match ite.next() {
                    Some(&arg) => {
                        context = match context.strip_prefix(arg) {
                            Some(context) => {
                                if context.is_empty() || context.starts_with(char::is_whitespace) {
                                    context.trim_start()
                                } else {
                                    return false; // context mismatch (wrong context)
                                }
                            }
                            None => {
                                return false; // context mismatch (wrong context)
                            }
                        }
                    }
                    None => {
                        return false; // context mismatch (too few arguments)
                    }
                }
                if context.is_empty() {
                    break;
                }
            }
            self.global || ite.next() == None
        }
    }
}

impl Trigger {
    pub fn matches(&self, last_arg: &str) -> Result<bool, regex::Error> {
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
    pub fn is_match(&self, args_until_last: &[&str], last_arg: &str) -> bool {
        self.context.matches(args_until_last)
            && self.trigger.matches(last_arg).unwrap_or_else(|err| {
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
    fn test_matches_context() {
        struct Scenario {
            pub testname: &'static str,
            pub abbr_context: Context,
            pub args_until_last: Vec<&'static str>,
            pub expected: bool,
        }
        let scenarios = &[
            Scenario {
                testname: r#"should match if empty context, non-global, args == ""#,
                abbr_context: Context {
                    context: String::new(),
                    global: false,
                },
                args_until_last: vec![],
                expected: true,
            },
            Scenario {
                testname: r#"should not match if empty context, non-global, args == "a""#,
                abbr_context: Context {
                    context: String::new(),
                    global: false,
                },
                args_until_last: vec!["a"],
                expected: false,
            },
            Scenario {
                testname: r#"should match if empty context, global, args == """#,
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                args_until_last: vec![],
                expected: true,
            },
            Scenario {
                testname: r#"should match if empty context, global, args == "a""#,
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                args_until_last: vec!["a"],
                expected: true,
            },
            Scenario {
                testname: r#"should not match if context == "git", non-global, args == """#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec![],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git", non-global, args == "a""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec!["a"],
                expected: false,
            },
            Scenario {
                testname: r#"should match if context == "git", non-global, args =="git""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec!["git"],
                expected: true,
            },
            Scenario {
                testname: r#"should not match if context == "git", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git", global, args == """#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec![],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git", global, args == "a""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["a"],
                expected: false,
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "git""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["git"],
                expected: true,
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "git commit""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit"],
                expected: true,
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "echo git""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["echo", "git"],
                expected: false,
            },
            Scenario {
                testname: r#"should match if context == "git commit", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: true,
            },
            Scenario {
                testname: r#"should match if context == "  git  commit  ", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "  git  commit  ".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: true,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git"],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == """#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec![""],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git commit -m""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit", "-m"],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git commita""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commita"],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git com""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "com"],
                expected: false,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", global, args == "git""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git"],
                expected: false,
            },
            Scenario {
                testname: r#"should match if context == "git commit", global, args == "git commit""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit"],
                expected: true,
            },
            Scenario {
                testname: r#"should match if context == "git commit", global, args == "git commit -m""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit", "-m"],
                expected: true,
            },
        ];
        for s in scenarios {
            assert_eq!(
                s.abbr_context.matches(&s.args_until_last),
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
            assert_eq!(s.trigger.matches(&s.last_arg), s.expected, "{}", s.testname);
        }
    }
}

fn default_as_false() -> bool {
    false
}

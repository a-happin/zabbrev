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
    #[serde(rename = "abbr-suffix")]
    AbbrSuffix(String),
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

#[derive(Debug, PartialEq)]
struct ContextMatchResult {
    pub context_size: usize,
}

#[derive(Debug)]
pub struct MatchResult <'a> {
    pub abbrev: &'a Abbrev,
    pub context_size: usize,
}

impl Context {
    fn matches(&self, args_until_last: &[&str]) -> Option<ContextMatchResult> {
        let mut context = self.context.trim_start();
        let mut i = 0;
        while !context.is_empty() {
            let &arg = args_until_last.get(i)?; // return because of too few arguments
            context = context.strip_prefix(arg).and_then (|context|
                if context.is_empty() || context.starts_with(char::is_whitespace) {
                    Some(context.trim_start())
                } else {
                    None // context mismatch (wrong context)
                }
            )?; // return because of context mismatch
            i += 1;
        }
        if self.global || args_until_last.get (i) == None
        {
            Some(ContextMatchResult {context_size: i})
        }
        else
        {
            None
        }
    }
}

impl Trigger {
    pub fn matches(&self, last_arg: &str) -> Result<bool, regex::Error> {
        match self {
            Self::AbbrString(ref abbr) => Ok(last_arg == abbr),
            Self::AbbrSuffix(ref suffix) => Ok(match last_arg.strip_suffix(suffix) {
                Some(last_arg) => last_arg.ends_with("."),
                None => false,
            }),
            Self::AbbrRegex(ref regex) => {
                let pattern = Regex::new(regex)?;
                Ok(pattern.is_match(last_arg))
            }
        }
    }
}

impl Abbrev {
    pub fn matches(&self, args_until_last: &[&str], last_arg: &str) -> Option<MatchResult> {
        let ContextMatchResult {context_size} = self.context.matches(args_until_last)?;
        match self.trigger.matches(last_arg) {
            Ok(true) => Some(MatchResult {abbrev: self, context_size}),
            Ok(false) => None,
            Err(err)=> {
                let name = self.name.as_ref().unwrap_or(&self.snippet);
                let error_message = format!("invalid regex in abbrev '{}': {}", name, err);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                None
            },
        }
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
            pub expected: Option<ContextMatchResult>,
        }
        let scenarios = &[
            Scenario {
                testname: r#"should match if empty context, non-global, args == ""#,
                abbr_context: Context {
                    context: String::new(),
                    global: false,
                },
                args_until_last: vec![],
                expected: Some(ContextMatchResult{context_size: 0}),
            },
            Scenario {
                testname: r#"should not match if empty context, non-global, args == "a""#,
                abbr_context: Context {
                    context: String::new(),
                    global: false,
                },
                args_until_last: vec!["a"],
                expected: None,
            },
            Scenario {
                testname: r#"should match if empty context, global, args == """#,
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                args_until_last: vec![],
                expected: Some(ContextMatchResult{context_size: 0}),
            },
            Scenario {
                testname: r#"should match if empty context, global, args == "a""#,
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                args_until_last: vec!["a"],
                expected: Some(ContextMatchResult{context_size: 0}),
            },
            Scenario {
                testname: r#"should not match if context == "git", non-global, args == """#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec![],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git", non-global, args == "a""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec!["a"],
                expected: None,
            },
            Scenario {
                testname: r#"should match if context == "git", non-global, args =="git""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec!["git"],
                expected: Some(ContextMatchResult{context_size: 1}),
            },
            Scenario {
                testname: r#"should not match if context == "git", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git", global, args == """#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec![],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git", global, args == "a""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["a"],
                expected: None,
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "git""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["git"],
                expected: Some(ContextMatchResult{context_size: 1}),
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "git commit""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit"],
                expected: Some(ContextMatchResult{context_size: 1}),
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "echo git""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["echo", "git"],
                expected: None,
            },
            Scenario {
                testname: r#"should match if context == "git commit", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: Some(ContextMatchResult{context_size: 2}),
            },
            Scenario {
                testname: r#"should match if context == "  git  commit  ", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "  git  commit  ".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: Some(ContextMatchResult{context_size: 2}),
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git"],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == """#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec![""],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git commit -m""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit", "-m"],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git commita""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commita"],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", non-global, args == "git com""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "com"],
                expected: None,
            },
            Scenario {
                testname: r#"should not match if context == "git commit", global, args == "git""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git"],
                expected: None,
            },
            Scenario {
                testname: r#"should match if context == "git commit", global, args == "git commit""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit"],
                expected: Some(ContextMatchResult{context_size: 2}),
            },
            Scenario {
                testname: r#"should match if context == "git commit", global, args == "git commit -m""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit", "-m"],
                expected: Some(ContextMatchResult{context_size: 2}),
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
    fn test_matches_trigger() {
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
                testname: "should match suffix",
                trigger: Trigger::AbbrSuffix("test".to_string()),
                last_arg: "a.test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should match suffix",
                trigger: Trigger::AbbrSuffix("test".to_string()),
                last_arg: ".test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should not match suffix",
                trigger: Trigger::AbbrSuffix("test".to_string()),
                last_arg: "test",
                expected: Ok(false),
            },
            Scenario {
                testname: "should not match suffix",
                trigger: Trigger::AbbrSuffix("test".to_string()),
                last_arg: "atest",
                expected: Ok(false),
            },
            Scenario {
                testname: "should match regex",
                trigger: Trigger::AbbrRegex(".+".to_string()),
                last_arg: "test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should not match regex",
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

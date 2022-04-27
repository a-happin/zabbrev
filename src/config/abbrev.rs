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
impl Context {
    fn matches(&self, args_until_last: &[&str]) -> Option<ContextMatchResult> {
        let mut context = self.context.trim_start();
        let mut i = 0;
        while !context.is_empty() {
            let &arg = args_until_last.get(i)?; // return because of too few arguments
            context = context.strip_prefix(arg).and_then(|context| {
                if context.is_empty() || context.starts_with(char::is_whitespace) {
                    Some(context.trim_start())
                } else {
                    None // context mismatch (wrong context)
                }
            })?; // return because of context mismatch
            i += 1;
        }
        if self.global || args_until_last.get(i) == None {
            Some(ContextMatchResult { context_size: i })
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
struct ContextMatchResult {
    pub context_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Trigger {
    #[serde(rename = "abbr")]
    AbbrString(String),
    #[serde(rename = "abbr-suffix")]
    AbbrSuffix(String),
    #[serde(rename = "abbr-prefix")]
    AbbrPrefix(String),
    #[serde(rename = "abbr-regex")]
    AbbrRegex(String),
}
impl Trigger {
    pub fn get_abbr(&self) -> &str {
        match self {
            Self::AbbrString(ref abbr) => abbr,
            Self::AbbrSuffix(ref suffix) => suffix,
            Self::AbbrPrefix(ref prefix) => prefix,
            Self::AbbrRegex(ref regex) => regex,
        }
    }
    pub fn matches(&self, last_arg: &str) -> Result<bool, regex::Error> {
        match self {
            Self::AbbrString(ref abbr) => Ok(last_arg == abbr),
            Self::AbbrSuffix(ref suffix) => Ok(last_arg.ends_with(suffix)),
            Self::AbbrPrefix(ref prefix) => Ok(last_arg.starts_with(prefix)),
            Self::AbbrRegex(ref regex) => {
                let pattern = Regex::new(regex)?;
                Ok(pattern.is_match(last_arg))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    #[serde(rename = "replace-self")]
    ReplaceSelf,
    #[serde(rename = "replace-context")]
    ReplaceContext,
    #[serde(rename = "replace-all")]
    ReplaceAll,
    #[serde(rename = "append")]
    Append,
    #[serde(rename = "prepend")]
    Prepend,
}
impl Default for Operation {
    fn default() -> Self {
        Self::ReplaceSelf
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub snippet: String,

    #[serde(default)]
    pub operation: Operation,

    pub cursor: Option<String>,

    #[serde(default = "default_as_false")]
    pub evaluate: bool,

    #[serde(default = "default_as_false")]
    pub redraw: bool,
}
impl Function {
    #[inline]
    #[allow(dead_code)]
    fn get_snippet(&self) -> Snippet {
        Snippet::new(&self.snippet, self.cursor.as_deref())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Abbrev {
    pub name: Option<String>,

    #[serde(flatten)]
    pub context: Context,

    #[serde(flatten)]
    pub trigger: Trigger,

    #[serde(flatten)]
    pub function: Function,
}

impl Abbrev {
    pub fn matches(&self, args_until_last: &[&str], last_arg: &str) -> Option<MatchResult> {
        let ContextMatchResult { context_size } = self.context.matches(args_until_last)?;
        match self.trigger.matches(last_arg) {
            Ok(true) => Some(MatchResult {
                abbrev: self,
                context_size,
            }),
            Ok(false) => None,
            Err(err) => {
                let error_message =
                    format!("invalid regex in abbrev '{}': {}", self.get_name(), err);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                None
            }
        }
    }
    #[inline]
    pub fn get_name(&self) -> &str {
        match self.name {
            Some(ref name) => name,
            None => &self.function.snippet,
        }
    }
}

#[derive(Debug)]
pub struct MatchResult<'a> {
    pub abbrev: &'a Abbrev,
    pub context_size: usize,
}

#[derive(Debug, PartialEq)]
pub enum Snippet<'a> {
    Simple(&'a str),
    Divided(&'a str, &'a str),
}
impl<'a> Snippet<'a> {
    #[inline]
    fn new_impl(snippet_string: &'a str, cursor: Option<&str>) -> Option<Snippet<'a>> {
        let cursor = cursor?;
        let (first, second) = snippet_string.split_once(cursor)?;
        Some(Self::Divided(first, second))
    }
    #[inline]
    pub fn new(snippet_string: &'a str, cursor: Option<&str>) -> Self {
        Snippet::new_impl(snippet_string, cursor).unwrap_or_else(|| Self::Simple(snippet_string))
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
                expected: Some(ContextMatchResult { context_size: 0 }),
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
                expected: Some(ContextMatchResult { context_size: 0 }),
            },
            Scenario {
                testname: r#"should match if empty context, global, args == "a""#,
                abbr_context: Context {
                    context: String::new(),
                    global: true,
                },
                args_until_last: vec!["a"],
                expected: Some(ContextMatchResult { context_size: 0 }),
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
                expected: Some(ContextMatchResult { context_size: 1 }),
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
                expected: Some(ContextMatchResult { context_size: 1 }),
            },
            Scenario {
                testname: r#"should match if context == "git", global, args == "git commit""#,
                abbr_context: Context {
                    context: "git".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit"],
                expected: Some(ContextMatchResult { context_size: 1 }),
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
                expected: Some(ContextMatchResult { context_size: 2 }),
            },
            Scenario {
                testname: r#"should match if context == "  git  commit  ", non-global, args == "git commit""#,
                abbr_context: Context {
                    context: "  git  commit  ".to_string(),
                    global: false,
                },
                args_until_last: vec!["git", "commit"],
                expected: Some(ContextMatchResult { context_size: 2 }),
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
                expected: Some(ContextMatchResult { context_size: 2 }),
            },
            Scenario {
                testname: r#"should match if context == "git commit", global, args == "git commit -m""#,
                abbr_context: Context {
                    context: "git commit".to_string(),
                    global: true,
                },
                args_until_last: vec!["git", "commit", "-m"],
                expected: Some(ContextMatchResult { context_size: 2 }),
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
                trigger: Trigger::AbbrSuffix(".test".to_string()),
                last_arg: "a.test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should match suffix",
                trigger: Trigger::AbbrSuffix(".test".to_string()),
                last_arg: ".test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should not match suffix",
                trigger: Trigger::AbbrSuffix(".test".to_string()),
                last_arg: "test",
                expected: Ok(false),
            },
            Scenario {
                testname: "should match prefix",
                trigger: Trigger::AbbrPrefix("test".to_string()),
                last_arg: "testa",
                expected: Ok(true),
            },
            Scenario {
                testname: "should match prefix",
                trigger: Trigger::AbbrPrefix("test".to_string()),
                last_arg: "test",
                expected: Ok(true),
            },
            Scenario {
                testname: "should not match prefix",
                trigger: Trigger::AbbrPrefix("test".to_string()),
                last_arg: "tes",
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

    #[test]
    fn test_divide_snippet() {
        struct Scenario {
            pub testname: &'static str,
            pub function: Function,
            pub expected: Snippet<'static>,
        }

        let scenarios = &[
            Scenario {
                testname: "no division",
                function: Function {
                    snippet: "[[ <> ]]".to_string(),
                    operation: Operation::ReplaceSelf,
                    cursor: None,
                    evaluate: false,
                    redraw: false,
                },
                expected: Snippet::Simple("[[ <> ]]"),
            },
            Scenario {
                testname: "division failed",
                function: Function {
                    snippet: "[[ <> ]]".to_string(),
                    operation: Operation::ReplaceSelf,
                    cursor: Some("🐣".to_string()),
                    evaluate: false,
                    redraw: false,
                },
                expected: Snippet::Simple("[[ <> ]]"),
            },
            Scenario {
                testname: "divide correctly",
                function: Function {
                    snippet: "[[ 🐣 ]]".to_string(),
                    operation: Operation::ReplaceSelf,
                    cursor: Some("🐣".to_string()),
                    evaluate: false,
                    redraw: false,
                },
                expected: Snippet::Divided("[[ ", " ]]"),
            },
        ];
        for s in scenarios {
            assert_eq!(s.function.get_snippet(), s.expected, "{}", s.testname);
        }
    }
}

fn default_as_false() -> bool {
    false
}

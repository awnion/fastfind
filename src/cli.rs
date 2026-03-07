use std::path::PathBuf;

/// Parsed and validated configuration for a find invocation.
#[derive(Debug, Clone)]
pub struct Config {
    pub paths: Vec<PathBuf>,
    pub max_depth: Option<usize>,
    pub min_depth: usize,
    pub file_type: Option<FileType>,
    pub name_pattern: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

impl Config {
    /// Parse a GNU find-style argument list.
    ///
    /// Supported:
    ///   [path...] [-maxdepth N] [-mindepth N] [-type f|d|l] [-name PATTERN] [-print]
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
        let mut paths = Vec::new();
        let mut max_depth = None;
        let mut min_depth = 0;
        let mut file_type = None;
        let mut name_pattern = None;

        let mut i = 0;
        // leading positional paths (before any flag)
        while i < args.len() && !args[i].starts_with('-') {
            paths.push(PathBuf::from(&args[i]));
            i += 1;
        }

        while i < args.len() {
            match args[i].as_str() {
                "-maxdepth" => {
                    i += 1;
                    let val = args.get(i).ok_or("-maxdepth requires an argument")?;
                    max_depth = Some(
                        val.parse::<usize>()
                            .map_err(|_| format!("-maxdepth: `{val}` is not a valid number"))?,
                    );
                }
                "-mindepth" => {
                    i += 1;
                    let val = args.get(i).ok_or("-mindepth requires an argument")?;
                    min_depth = val
                        .parse::<usize>()
                        .map_err(|_| format!("-mindepth: `{val}` is not a valid number"))?;
                }
                "-type" => {
                    i += 1;
                    let val = args.get(i).ok_or("-type requires an argument")?;
                    file_type = Some(match val.as_str() {
                        "f" => FileType::File,
                        "d" => FileType::Directory,
                        "l" => FileType::Symlink,
                        other => return Err(format!("-type: unknown type `{other}`")),
                    });
                }
                "-name" => {
                    i += 1;
                    let val = args.get(i).ok_or("-name requires an argument")?;
                    name_pattern = Some(val.clone());
                }
                "-print" => {
                    // default action, accepted but no-op
                }
                "--version" | "-version" => {
                    return Err("__version__".into());
                }
                "--help" | "-help" => {
                    return Err("__help__".into());
                }
                other => {
                    return Err(format!("unknown option: `{other}`"));
                }
            }
            i += 1;
        }

        if paths.is_empty() {
            paths.push(PathBuf::from("."));
        }

        Ok(Config { paths, max_depth, min_depth, file_type, name_pattern })
    }
}

/// Convert a shell glob pattern to a regex pattern.
/// Supports *, ?, and [...] character classes.
pub fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '[' => {
                regex.push('[');
                i += 1;
                // pass through bracket contents verbatim
                while i < chars.len() && chars[i] != ']' {
                    if chars[i] == '\\' {
                        regex.push('\\');
                        i += 1;
                        if i < chars.len() {
                            regex.push(chars[i]);
                        }
                    } else {
                        regex.push(chars[i]);
                    }
                    i += 1;
                }
                regex.push(']');
            }
            '.' | '+' | '(' | ')' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                regex.push('\\');
                regex.push(chars[i]);
            }
            c => regex.push(c),
        }
        i += 1;
    }
    regex.push('$');
    regex
}

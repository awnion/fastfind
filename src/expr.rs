use std::path::PathBuf;
use std::time::SystemTime;

use regex::Regex;

/// Numeric comparison: +n, -n, or n
#[derive(Debug, Clone, Copy)]
pub enum Cmp {
    Greater, // +n
    Equal,   // n
    Less,    // -n
}

impl Cmp {
    #[inline]
    pub fn matches(self, actual: i64, target: i64) -> bool {
        match self {
            Cmp::Greater => actual > target,
            Cmp::Less => actual < target,
            Cmp::Equal => actual == target,
        }
    }

    #[inline]
    pub fn matches_u64(self, actual: u64, target: u64) -> bool {
        match self {
            Cmp::Greater => actual > target,
            Cmp::Less => actual < target,
            Cmp::Equal => actual == target,
        }
    }
}

/// Parse +n, -n, or n from a string
pub fn parse_cmp(s: &str) -> Result<(Cmp, i64), String> {
    if let Some(rest) = s.strip_prefix('+') {
        let n: i64 = rest.parse().map_err(|_| format!("invalid number: `{s}`"))?;
        Ok((Cmp::Greater, n))
    } else if s.starts_with('-') {
        let n: i64 = s.parse().map_err(|_| format!("invalid number: `{s}`"))?;
        Ok((Cmp::Less, n.abs()))
    } else {
        let n: i64 = s.parse().map_err(|_| format!("invalid number: `{s}`"))?;
        Ok((Cmp::Equal, n))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    BlockDevice, // b
    CharDevice,  // c
    Directory,   // d
    Pipe,        // p
    File,        // f
    Symlink,     // l
    Socket,      // s
}

impl FileType {
    pub fn from_char(c: char) -> Result<Self, String> {
        match c {
            'b' => Ok(FileType::BlockDevice),
            'c' => Ok(FileType::CharDevice),
            'd' => Ok(FileType::Directory),
            'p' => Ok(FileType::Pipe),
            'f' => Ok(FileType::File),
            'l' => Ok(FileType::Symlink),
            's' => Ok(FileType::Socket),
            _ => Err(format!("unknown file type `{c}`")),
        }
    }

    pub fn to_char(self) -> char {
        match self {
            FileType::BlockDevice => 'b',
            FileType::CharDevice => 'c',
            FileType::Directory => 'd',
            FileType::Pipe => 'p',
            FileType::File => 'f',
            FileType::Symlink => 'l',
            FileType::Socket => 's',
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SizeUnit {
    Bytes,     // c
    Words,     // w (2 bytes)
    Blocks512, // b or default (512 bytes)
    Kilobytes, // k (1024)
    Megabytes, // M (1048576)
    Gigabytes, // G (1073741824)
}

impl SizeUnit {
    pub fn bytes(self) -> u64 {
        match self {
            SizeUnit::Bytes => 1,
            SizeUnit::Words => 2,
            SizeUnit::Blocks512 => 512,
            SizeUnit::Kilobytes => 1024,
            SizeUnit::Megabytes => 1_048_576,
            SizeUnit::Gigabytes => 1_073_741_824,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PermMatch {
    Exact, // -perm mode
    All,   // -perm -mode
    Any,   // -perm /mode
}

#[derive(Debug, Clone, Copy)]
pub enum TimeRef {
    Access, // a
    Modify, // m (or Birth on some systems)
    Change, // c
    Birth,  // B
}

/// Compiled glob pattern with fast-path optimization for common patterns
#[derive(Debug, Clone)]
pub struct GlobPattern {
    strategy: MatchStrategy,
}

#[derive(Debug, Clone)]
enum MatchStrategy {
    /// Fast path: pattern is `*suffix` (e.g., `*.rs`)
    Suffix(String),
    /// Fast path: exact literal match
    Exact(String),
    /// Fast path: prefix match (`prefix*`)
    Prefix(String),
    /// General regex fallback
    Regex(Regex),
}

impl GlobPattern {
    pub fn new(pattern: &str, case_insensitive: bool) -> Result<Self, String> {
        // Try fast-path optimizations for common patterns (only case-sensitive)
        if !case_insensitive {
            // `*.ext` -> suffix match
            if let Some(suffix) = pattern.strip_prefix('*')
                && !suffix.contains(['*', '?', '[', ']'])
            {
                return Ok(GlobPattern { strategy: MatchStrategy::Suffix(suffix.to_string()) });
            }
            // Exact literal (no glob chars)
            if !pattern.contains(['*', '?', '[', ']']) {
                return Ok(GlobPattern { strategy: MatchStrategy::Exact(pattern.to_string()) });
            }
            // `prefix*` -> prefix match
            if pattern.ends_with('*')
                && !pattern[..pattern.len() - 1].contains(['*', '?', '[', ']'])
            {
                return Ok(GlobPattern {
                    strategy: MatchStrategy::Prefix(pattern[..pattern.len() - 1].to_string()),
                });
            }
        }

        // Fallback to regex
        let regex_str = glob_to_regex(pattern);
        let full = if case_insensitive { format!("(?i){regex_str}") } else { regex_str };
        let re = Regex::new(&full).map_err(|e| format!("invalid pattern: {e}"))?;
        Ok(GlobPattern { strategy: MatchStrategy::Regex(re) })
    }

    #[inline]
    pub fn is_match(&self, s: &str) -> bool {
        match &self.strategy {
            MatchStrategy::Suffix(suffix) => s.ends_with(suffix.as_str()),
            MatchStrategy::Exact(exact) => s == exact.as_str(),
            MatchStrategy::Prefix(prefix) => s.starts_with(prefix.as_str()),
            MatchStrategy::Regex(re) => re.is_match(s),
        }
    }

    /// Match against raw bytes, avoiding UTF-8 validation for fast paths.
    #[inline]
    pub fn is_match_bytes(&self, bytes: &[u8]) -> bool {
        match &self.strategy {
            MatchStrategy::Suffix(suffix) => bytes.ends_with(suffix.as_bytes()),
            MatchStrategy::Exact(exact) => bytes == exact.as_bytes(),
            MatchStrategy::Prefix(prefix) => bytes.starts_with(prefix.as_bytes()),
            MatchStrategy::Regex(re) => {
                // Regex needs a str; fall back to UTF-8 conversion
                match std::str::from_utf8(bytes) {
                    Ok(s) => re.is_match(s),
                    Err(_) => false,
                }
            }
        }
    }
}

/// Convert a shell glob pattern to a regex pattern.
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
                // handle negation with !
                if i < chars.len() && chars[i] == '!' {
                    regex.push('^');
                    i += 1;
                }
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

/// Argument to -exec/-execdir commands
#[derive(Debug, Clone)]
pub enum ExecArg {
    Literal(String),
    Placeholder, // {}
}

/// Printf format token
#[derive(Debug, Clone)]
pub enum PrintfToken {
    Literal(String),
    Percent,               // %%
    Path,                  // %p
    Filename,              // %f
    ParentDir,             // %h
    Depth,                 // %d
    Size,                  // %s
    SizeInBlocks,          // %k (1K blocks)
    BlockSize,             // %b (512-byte blocks)
    Permissions,           // %m (octal)
    PermSymbolic,          // %M (symbolic like ls)
    Nlinks,                // %n
    User,                  // %u
    Group,                 // %g
    Uid,                   // %U
    Gid,                   // %G
    Inode,                 // %i
    DeviceNumber,          // %D
    Type,                  // %y (type char)
    LinkTarget,            // %l
    TimeAccess,            // %a (ctime format)
    TimeAccessSecs,        // %A@ (epoch secs)
    TimeModify,            // %t (ctime format)
    TimeModifySecs,        // %T@ (epoch secs)
    TimeChange,            // %c (ctime format)
    TimeChangeSecs,        // %C@ (epoch secs)
    Newline,               // \n
    Tab,                   // \t
    NullChar,              // \0
    Backslash,             // \\
    TimeAccessFmt(String), // %A with strftime format
    TimeModifyFmt(String), // %T with strftime format
    TimeChangeFmt(String), // %C with strftime format
    SparseName,            // %P (path relative to starting point)
    StartingPoint,         // %H (starting point)
}

/// The expression tree
#[derive(Debug)]
pub enum Expr {
    // Operators
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    List(Vec<Expr>), // comma operator

    // Tests
    True,
    False,
    Name(GlobPattern),
    IName(GlobPattern),
    Path(GlobPattern),
    IPath(GlobPattern),
    LName(GlobPattern),
    ILName(GlobPattern),
    Regex(Regex),
    IRegex(Regex),
    Type(Vec<FileType>),
    XType(Vec<FileType>),
    Size {
        cmp: Cmp,
        size: u64,
        unit: SizeUnit,
    },
    Empty,
    Perm {
        mode: u32,
        match_type: PermMatch,
    },
    User(String),
    Group(String),
    Uid(Cmp, u64),
    Gid(Cmp, u64),
    NoUser,
    NoGroup,
    Links(Cmp, u64),
    Inum(Cmp, u64),
    SameFile {
        dev: u64,
        ino: u64,
    },
    Readable,
    Writable,
    Executable,
    MTime(Cmp, i64),
    MMin(Cmp, i64),
    ATime(Cmp, i64),
    AMin(Cmp, i64),
    CTime(Cmp, i64),
    CMin(Cmp, i64),
    Newer(SystemTime),
    ANewerM(SystemTime),
    CNewerM(SystemTime),
    NewerXY {
        x: TimeRef,
        y: TimeRef,
        reference: SystemTime,
    },
    Used(Cmp, i64),
    #[cfg(unix)]
    FsType(String),

    // Actions
    Print,
    Print0,
    Printf(Vec<PrintfToken>),
    Ls,
    FPrint(PathBuf),
    FPrint0(PathBuf),
    FPrintf(PathBuf, Vec<PrintfToken>),
    FLs(PathBuf),
    Exec {
        args: Vec<ExecArg>,
        batch: bool,
    },
    ExecDir {
        args: Vec<ExecArg>,
        batch: bool,
    },
    Ok {
        args: Vec<ExecArg>,
    },
    OkDir {
        args: Vec<ExecArg>,
    },
    Delete,
    Prune,
    Quit,
}

/// Check if an expression tree contains any action
pub fn has_action(expr: &Expr) -> bool {
    match expr {
        Expr::And(exprs) | Expr::Or(exprs) | Expr::List(exprs) => exprs.iter().any(has_action),
        Expr::Not(e) => has_action(e),
        Expr::Print
        | Expr::Print0
        | Expr::Printf(_)
        | Expr::Ls
        | Expr::FPrint(_)
        | Expr::FPrint0(_)
        | Expr::FPrintf(_, _)
        | Expr::FLs(_)
        | Expr::Exec { .. }
        | Expr::ExecDir { .. }
        | Expr::Ok { .. }
        | Expr::OkDir { .. }
        | Expr::Delete
        | Expr::Quit => true,
        Expr::Prune => false, // prune is special - it's a test that has a side effect
        _ => false,
    }
}

/// Check if an expression tree contains -prune
pub fn has_prune(expr: &Expr) -> bool {
    match expr {
        Expr::And(exprs) | Expr::Or(exprs) | Expr::List(exprs) => exprs.iter().any(has_prune),
        Expr::Not(e) => has_prune(e),
        Expr::Prune => true,
        _ => false,
    }
}

/// Symlink following mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkMode {
    Never,       // -P (default)
    Always,      // -L
    CommandLine, // -H
}

/// Regex dialect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexType {
    Emacs, // default for GNU find
    PosixBasic,
    PosixExtended,
    Grep,
    Egrep,
}

/// Global configuration
#[derive(Debug)]
pub struct Config {
    pub paths: Vec<PathBuf>,
    pub max_depth: Option<usize>,
    pub min_depth: usize,
    pub depth_first: bool, // -depth/-d
    pub xdev: bool,        // -xdev/-mount
    pub symlink_mode: SymlinkMode,
    pub daystart: bool,
    pub regex_type: RegexType,
    pub warn: bool,
    pub noleaf: bool,
    pub ignore_readdir_race: bool,
    pub expr: Expr,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            paths: vec![PathBuf::from(".")],
            max_depth: None,
            min_depth: 0,
            depth_first: false,
            xdev: false,
            symlink_mode: SymlinkMode::Never,
            daystart: false,
            regex_type: RegexType::Emacs,
            warn: true,
            noleaf: false,
            ignore_readdir_race: false,
            expr: Expr::True,
        }
    }
}

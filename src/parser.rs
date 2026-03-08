use std::path::PathBuf;
use std::time::SystemTime;

use regex::Regex;

use crate::expr::*;

/// Special error types for --version and --help
pub const VERSION_SENTINEL: &str = "__version__";
pub const HELP_SENTINEL: &str = "__help__";

/// Parse GNU find-style arguments into a Config.
pub fn parse<I, S>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    let mut config = Config::default();
    let mut i = 0;

    // Parse leading options: -H, -L, -P, -D, -O
    while i < args.len() {
        match args[i].as_str() {
            "-H" => config.symlink_mode = SymlinkMode::CommandLine,
            "-L" => config.symlink_mode = SymlinkMode::Always,
            "-P" => config.symlink_mode = SymlinkMode::Never,
            _ => break,
        }
        i += 1;
    }

    // Parse paths (before any expression)
    let mut paths = Vec::new();
    while i < args.len() {
        let arg = &args[i];
        // Paths are arguments that don't start with - and aren't ( or !
        if arg.starts_with('-') || arg == "(" || arg == ")" || arg == "!" || arg == "," {
            break;
        }
        paths.push(PathBuf::from(arg));
        i += 1;
    }

    if !paths.is_empty() {
        config.paths = paths;
    }

    // Parse expression (may be empty = default to just -print)
    if i >= args.len() {
        config.expr = Expr::Print;
    } else {
        let (expr, has_action_flag) = parse_expression(&args, &mut i, &mut config)?;

        if i < args.len() {
            return Err(format!("unexpected argument: `{}`", args[i]));
        }

        // If no action in expression, add implicit -print:
        // (user_expr) -a -print
        if has_action_flag {
            config.expr = expr;
        } else {
            config.expr = Expr::And(vec![expr, Expr::Print]);
        }
    }

    Ok(config)
}

/// Parse an expression (handles -o / -or at the top level)
fn parse_expression(
    args: &[String],
    i: &mut usize,
    config: &mut Config,
) -> Result<(Expr, bool), String> {
    parse_or_expr(args, i, config)
}

/// Parse OR expression: expr1 -o expr2
fn parse_or_expr(
    args: &[String],
    i: &mut usize,
    config: &mut Config,
) -> Result<(Expr, bool), String> {
    let (first, mut has_act) = parse_list_expr(args, i, config)?;
    let mut exprs = vec![first];

    while *i < args.len() && matches!(args[*i].as_str(), "-o" | "-or") {
        *i += 1;
        let (next, act) = parse_list_expr(args, i, config)?;
        has_act = has_act || act;
        exprs.push(next);
    }

    if exprs.len() == 1 {
        Ok((exprs.pop().unwrap(), has_act))
    } else {
        Ok((Expr::Or(exprs), has_act))
    }
}

/// Parse comma (list) expression: expr1 , expr2
fn parse_list_expr(
    args: &[String],
    i: &mut usize,
    config: &mut Config,
) -> Result<(Expr, bool), String> {
    let (first, mut has_act) = parse_and_expr(args, i, config)?;
    let mut exprs = vec![first];

    while *i < args.len() && args[*i] == "," {
        *i += 1;
        let (next, act) = parse_and_expr(args, i, config)?;
        has_act = has_act || act;
        exprs.push(next);
    }

    if exprs.len() == 1 {
        Ok((exprs.pop().unwrap(), has_act))
    } else {
        Ok((Expr::List(exprs), has_act))
    }
}

/// Parse AND expression: expr1 [-a] expr2
fn parse_and_expr(
    args: &[String],
    i: &mut usize,
    config: &mut Config,
) -> Result<(Expr, bool), String> {
    let (first, mut has_act) = parse_unary_expr(args, i, config)?;
    let mut exprs = vec![first];

    loop {
        // Explicit -a / -and
        if *i < args.len() && matches!(args[*i].as_str(), "-a" | "-and") {
            *i += 1;
        }
        // Implicit AND: next token is not -o, -or, ), or ,
        if *i >= args.len() {
            break;
        }
        match args[*i].as_str() {
            "-o" | "-or" | ")" | "," => break,
            _ => {}
        }
        let (next, act) = parse_unary_expr(args, i, config)?;
        has_act = has_act || act;
        exprs.push(next);
    }

    if exprs.len() == 1 {
        Ok((exprs.pop().unwrap(), has_act))
    } else {
        Ok((Expr::And(exprs), has_act))
    }
}

/// Parse unary expression: ! expr or ( expr )
fn parse_unary_expr(
    args: &[String],
    i: &mut usize,
    config: &mut Config,
) -> Result<(Expr, bool), String> {
    if *i >= args.len() {
        return Err("expected expression".to_string());
    }

    match args[*i].as_str() {
        "!" | "-not" => {
            *i += 1;
            let (expr, act) = parse_unary_expr(args, i, config)?;
            Ok((Expr::Not(Box::new(expr)), act))
        }
        "(" => {
            *i += 1;
            let (expr, act) = parse_or_expr(args, i, config)?;
            if *i >= args.len() || args[*i] != ")" {
                return Err("missing closing `)`".to_string());
            }
            *i += 1;
            Ok((expr, act))
        }
        _ => parse_primary(args, i, config),
    }
}

/// Parse a primary expression (test, action, or option)
fn parse_primary(
    args: &[String],
    i: &mut usize,
    config: &mut Config,
) -> Result<(Expr, bool), String> {
    if *i >= args.len() {
        return Err("expected expression".to_string());
    }

    let arg = args[*i].clone();
    *i += 1;

    match arg.as_str() {
        // Version / help
        "--version" | "-version" => Err(VERSION_SENTINEL.to_string()),
        "--help" | "-help" => Err(HELP_SENTINEL.to_string()),

        // Global options (modify config, return True)
        "-maxdepth" => {
            let val = next_arg(args, i, "-maxdepth")?;
            config.max_depth = Some(
                val.parse::<usize>()
                    .map_err(|_| format!("-maxdepth: `{val}` is not a valid number"))?,
            );
            Ok((Expr::True, false))
        }
        "-mindepth" => {
            let val = next_arg(args, i, "-mindepth")?;
            config.min_depth = val
                .parse::<usize>()
                .map_err(|_| format!("-mindepth: `{val}` is not a valid number"))?;
            Ok((Expr::True, false))
        }
        "-depth" | "-d" => {
            config.depth_first = true;
            Ok((Expr::True, false))
        }
        "-xdev" | "-mount" => {
            config.xdev = true;
            Ok((Expr::True, false))
        }
        "-noleaf" => {
            config.noleaf = true;
            Ok((Expr::True, false))
        }
        "-ignore_readdir_race" => {
            config.ignore_readdir_race = true;
            Ok((Expr::True, false))
        }
        "-noignore_readdir_race" => {
            config.ignore_readdir_race = false;
            Ok((Expr::True, false))
        }
        "-warn" => {
            config.warn = true;
            Ok((Expr::True, false))
        }
        "-nowarn" => {
            config.warn = false;
            Ok((Expr::True, false))
        }
        "-daystart" => {
            config.daystart = true;
            Ok((Expr::True, false))
        }
        "-regextype" => {
            let val = next_arg(args, i, "-regextype")?;
            config.regex_type = match val.as_str() {
                "emacs" => RegexType::Emacs,
                "posix-basic" => RegexType::PosixBasic,
                "posix-extended" => RegexType::PosixExtended,
                "grep" => RegexType::Grep,
                "egrep" => RegexType::Egrep,
                other => return Err(format!("-regextype: unknown type `{other}`")),
            };
            Ok((Expr::True, false))
        }

        // Tests
        "-true" => Ok((Expr::True, false)),
        "-false" => Ok((Expr::False, false)),

        "-name" => {
            let pattern = next_arg(args, i, "-name")?;
            Ok((Expr::Name(GlobPattern::new(&pattern, false)?), false))
        }
        "-iname" => {
            let pattern = next_arg(args, i, "-iname")?;
            Ok((Expr::IName(GlobPattern::new(&pattern, true)?), false))
        }
        "-path" | "-wholename" => {
            let pattern = next_arg(args, i, &arg)?;
            Ok((Expr::Path(GlobPattern::new(&pattern, false)?), false))
        }
        "-ipath" | "-iwholename" => {
            let pattern = next_arg(args, i, &arg)?;
            Ok((Expr::IPath(GlobPattern::new(&pattern, true)?), false))
        }
        "-lname" => {
            let pattern = next_arg(args, i, "-lname")?;
            Ok((Expr::LName(GlobPattern::new(&pattern, false)?), false))
        }
        "-ilname" => {
            let pattern = next_arg(args, i, "-ilname")?;
            Ok((Expr::ILName(GlobPattern::new(&pattern, true)?), false))
        }
        "-regex" => {
            let pattern = next_arg(args, i, "-regex")?;
            let re = Regex::new(&pattern)
                .map_err(|e| format!("-regex: invalid regex `{pattern}`: {e}"))?;
            Ok((Expr::Regex(re), false))
        }
        "-iregex" => {
            let pattern = next_arg(args, i, "-iregex")?;
            let re = Regex::new(&format!("(?i){pattern}"))
                .map_err(|e| format!("-iregex: invalid regex `{pattern}`: {e}"))?;
            Ok((Expr::IRegex(re), false))
        }

        "-type" => {
            let val = next_arg(args, i, "-type")?;
            let types = parse_file_types(&val)?;
            Ok((Expr::Type(types), false))
        }
        "-xtype" => {
            let val = next_arg(args, i, "-xtype")?;
            let types = parse_file_types(&val)?;
            Ok((Expr::XType(types), false))
        }

        "-size" => {
            let val = next_arg(args, i, "-size")?;
            let (cmp, size, unit) = parse_size(&val)?;
            Ok((Expr::Size { cmp, size, unit }, false))
        }
        "-empty" => Ok((Expr::Empty, false)),

        "-perm" => {
            let val = next_arg(args, i, "-perm")?;
            let (mode, match_type) = parse_perm(&val)?;
            Ok((Expr::Perm { mode, match_type }, false))
        }
        "-readable" => Ok((Expr::Readable, false)),
        "-writable" => Ok((Expr::Writable, false)),
        "-executable" => Ok((Expr::Executable, false)),

        "-user" => {
            let val = next_arg(args, i, "-user")?;
            Ok((Expr::User(val), false))
        }
        "-group" => {
            let val = next_arg(args, i, "-group")?;
            Ok((Expr::Group(val), false))
        }
        "-uid" => {
            let val = next_arg(args, i, "-uid")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::Uid(cmp, n as u64), false))
        }
        "-gid" => {
            let val = next_arg(args, i, "-gid")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::Gid(cmp, n as u64), false))
        }
        "-nouser" => Ok((Expr::NoUser, false)),
        "-nogroup" => Ok((Expr::NoGroup, false)),

        "-links" => {
            let val = next_arg(args, i, "-links")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::Links(cmp, n as u64), false))
        }
        "-inum" => {
            let val = next_arg(args, i, "-inum")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::Inum(cmp, n as u64), false))
        }
        "-samefile" => {
            let val = next_arg(args, i, "-samefile")?;
            let path = PathBuf::from(&val);
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let meta = std::fs::metadata(&path)
                    .map_err(|e| format!("-samefile: cannot stat `{val}`: {e}"))?;
                Ok((Expr::SameFile { dev: meta.dev(), ino: meta.ino() }, false))
            }
            #[cfg(not(unix))]
            {
                let _ = path;
                Err("-samefile is not supported on this platform".to_string())
            }
        }

        "-mtime" => {
            let val = next_arg(args, i, "-mtime")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::MTime(cmp, n), false))
        }
        "-mmin" => {
            let val = next_arg(args, i, "-mmin")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::MMin(cmp, n), false))
        }
        "-atime" => {
            let val = next_arg(args, i, "-atime")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::ATime(cmp, n), false))
        }
        "-amin" => {
            let val = next_arg(args, i, "-amin")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::AMin(cmp, n), false))
        }
        "-ctime" => {
            let val = next_arg(args, i, "-ctime")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::CTime(cmp, n), false))
        }
        "-cmin" => {
            let val = next_arg(args, i, "-cmin")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::CMin(cmp, n), false))
        }
        "-newer" => {
            let val = next_arg(args, i, "-newer")?;
            let mtime = ref_mtime(&val)?;
            Ok((Expr::Newer(mtime), false))
        }
        "-anewer" => {
            let val = next_arg(args, i, "-anewer")?;
            let mtime = ref_mtime(&val)?;
            Ok((Expr::ANewerM(mtime), false))
        }
        "-cnewer" => {
            let val = next_arg(args, i, "-cnewer")?;
            let mtime = ref_mtime(&val)?;
            Ok((Expr::CNewerM(mtime), false))
        }
        s if s.starts_with("-newer") && s.len() >= 8 => {
            // -newerXY where X and Y are time ref chars
            let chars: Vec<char> = s[6..].chars().collect();
            if chars.len() != 2 {
                return Err(format!("unknown option: `{s}`"));
            }
            let x = parse_time_ref(chars[0])
                .ok_or_else(|| format!("-newerXY: unknown time reference `{}`", chars[0]))?;
            let y = parse_time_ref(chars[1])
                .ok_or_else(|| format!("-newerXY: unknown time reference `{}`", chars[1]))?;
            let val = next_arg(args, i, s)?;
            let reference = ref_time_of(&val, y)?;
            Ok((Expr::NewerXY { x, y, reference }, false))
        }
        "-used" => {
            let val = next_arg(args, i, "-used")?;
            let (cmp, n) = parse_cmp(&val)?;
            Ok((Expr::Used(cmp, n), false))
        }
        #[cfg(unix)]
        "-fstype" => {
            let val = next_arg(args, i, "-fstype")?;
            Ok((Expr::FsType(val), false))
        }

        // Actions
        "-print" => Ok((Expr::Print, true)),
        "-print0" => Ok((Expr::Print0, true)),
        "-printf" => {
            let fmt = next_arg(args, i, "-printf")?;
            let tokens = parse_printf(&fmt)?;
            Ok((Expr::Printf(tokens), true))
        }
        "-ls" => Ok((Expr::Ls, true)),
        "-fprint" => {
            let file = next_arg(args, i, "-fprint")?;
            Ok((Expr::FPrint(PathBuf::from(file)), true))
        }
        "-fprint0" => {
            let file = next_arg(args, i, "-fprint0")?;
            Ok((Expr::FPrint0(PathBuf::from(file)), true))
        }
        "-fprintf" => {
            let file = next_arg(args, i, "-fprintf")?;
            let fmt = next_arg(args, i, "-fprintf")?;
            let tokens = parse_printf(&fmt)?;
            Ok((Expr::FPrintf(PathBuf::from(file), tokens), true))
        }
        "-fls" => {
            let file = next_arg(args, i, "-fls")?;
            Ok((Expr::FLs(PathBuf::from(file)), true))
        }
        "-exec" => {
            let (exec_args, batch) = parse_exec_args(args, i, "-exec")?;
            Ok((Expr::Exec { args: exec_args, batch }, true))
        }
        "-execdir" => {
            let (exec_args, batch) = parse_exec_args(args, i, "-execdir")?;
            Ok((Expr::ExecDir { args: exec_args, batch }, true))
        }
        "-ok" => {
            let (exec_args, batch) = parse_exec_args(args, i, "-ok")?;
            if batch {
                return Err("-ok: only `;` terminator is supported, not `+`".to_string());
            }
            Ok((Expr::Ok { args: exec_args }, true))
        }
        "-okdir" => {
            let (exec_args, batch) = parse_exec_args(args, i, "-okdir")?;
            if batch {
                return Err("-okdir: only `;` terminator is supported, not `+`".to_string());
            }
            Ok((Expr::OkDir { args: exec_args }, true))
        }
        "-delete" => {
            config.depth_first = true; // -delete implies -depth
            Ok((Expr::Delete, true))
        }
        "-prune" => Ok((Expr::Prune, false)),
        "-quit" => Ok((Expr::Quit, true)),

        other => Err(format!("unknown option: `{other}`")),
    }
}

fn next_arg(args: &[String], i: &mut usize, opt: &str) -> Result<String, String> {
    if *i >= args.len() {
        return Err(format!("{opt} requires an argument"));
    }
    let val = args[*i].clone();
    *i += 1;
    Ok(val)
}

fn parse_file_types(s: &str) -> Result<Vec<FileType>, String> {
    s.split(',')
        .map(|part| {
            let part = part.trim();
            if part.len() != 1 {
                return Err(format!("-type: unknown type `{part}`"));
            }
            FileType::from_char(part.chars().next().unwrap())
        })
        .collect()
}

fn parse_size(s: &str) -> Result<(Cmp, u64, SizeUnit), String> {
    let (cmp_str, rest) = if let Some(r) = s.strip_prefix('+') {
        (Cmp::Greater, r)
    } else if let Some(r) = s.strip_prefix('-') {
        (Cmp::Less, r)
    } else {
        (Cmp::Equal, s)
    };

    let (num_str, unit) = if let Some(n) = rest.strip_suffix('c') {
        (n, SizeUnit::Bytes)
    } else if let Some(n) = rest.strip_suffix('w') {
        (n, SizeUnit::Words)
    } else if let Some(n) = rest.strip_suffix('b') {
        (n, SizeUnit::Blocks512)
    } else if let Some(n) = rest.strip_suffix('k') {
        (n, SizeUnit::Kilobytes)
    } else if let Some(n) = rest.strip_suffix('M') {
        (n, SizeUnit::Megabytes)
    } else if let Some(n) = rest.strip_suffix('G') {
        (n, SizeUnit::Gigabytes)
    } else {
        (rest, SizeUnit::Blocks512) // default is 512-byte blocks
    };

    let n: u64 = num_str.parse().map_err(|_| format!("-size: invalid size `{s}`"))?;

    Ok((cmp_str, n, unit))
}

fn parse_perm(s: &str) -> Result<(u32, PermMatch), String> {
    if let Some(rest) = s.strip_prefix('-') {
        let mode = parse_octal_mode(rest)?;
        Ok((mode, PermMatch::All))
    } else if let Some(rest) = s.strip_prefix('/') {
        let mode = parse_octal_mode(rest)?;
        Ok((mode, PermMatch::Any))
    } else {
        let mode = parse_octal_mode(s)?;
        Ok((mode, PermMatch::Exact))
    }
}

fn parse_octal_mode(s: &str) -> Result<u32, String> {
    // Try parsing as octal first
    u32::from_str_radix(s, 8).map_err(|_| format!("-perm: invalid mode `{s}`"))
}

fn ref_mtime(path: &str) -> Result<SystemTime, String> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map_err(|e| format!("cannot get modification time of `{path}`: {e}"))
}

fn ref_time_of(path: &str, time_ref: TimeRef) -> Result<SystemTime, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("cannot stat `{path}`: {e}"))?;

    match time_ref {
        TimeRef::Modify => {
            meta.modified().map_err(|e| format!("cannot get modification time: {e}"))
        }
        TimeRef::Access => meta.accessed().map_err(|e| format!("cannot get access time: {e}")),
        TimeRef::Change => {
            // On Unix, ctime is available via MetadataExt
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                use std::time::UNIX_EPOCH;
                Ok(UNIX_EPOCH + std::time::Duration::from_secs(meta.ctime() as u64))
            }
            #[cfg(not(unix))]
            {
                meta.modified().map_err(|e| format!("cannot get change time: {e}"))
            }
        }
        TimeRef::Birth => meta.created().map_err(|e| format!("cannot get birth time: {e}")),
    }
}

fn parse_time_ref(c: char) -> Option<TimeRef> {
    match c {
        'a' => Some(TimeRef::Access),
        'm' => Some(TimeRef::Modify),
        'c' => Some(TimeRef::Change),
        'B' | 'W' => Some(TimeRef::Birth),
        't' => Some(TimeRef::Modify), // -newermt: compare against literal time
        _ => None,
    }
}

fn parse_exec_args(
    args: &[String],
    i: &mut usize,
    opt: &str,
) -> Result<(Vec<ExecArg>, bool), String> {
    let mut exec_args = Vec::new();
    let mut batch = false;

    loop {
        if *i >= args.len() {
            return Err(format!("{opt}: missing terminator `;` or `+`"));
        }
        let arg = &args[*i];
        *i += 1;

        if arg == ";" {
            break;
        }
        if arg == "+" {
            batch = true;
            break;
        }
        if arg == "{}" {
            exec_args.push(ExecArg::Placeholder);
        } else {
            exec_args.push(ExecArg::Literal(arg.clone()));
        }
    }

    Ok((exec_args, batch))
}

fn parse_printf(fmt: &str) -> Result<Vec<PrintfToken>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' {
            i += 1;
            if i >= chars.len() {
                tokens.push(PrintfToken::Literal("\\".to_string()));
                break;
            }
            match chars[i] {
                'n' => tokens.push(PrintfToken::Newline),
                't' => tokens.push(PrintfToken::Tab),
                '0' => tokens.push(PrintfToken::NullChar),
                '\\' => tokens.push(PrintfToken::Backslash),
                'a' => tokens.push(PrintfToken::Literal("\x07".to_string())),
                'b' => tokens.push(PrintfToken::Literal("\x08".to_string())),
                'f' => tokens.push(PrintfToken::Literal("\x0C".to_string())),
                'r' => tokens.push(PrintfToken::Literal("\r".to_string())),
                'v' => tokens.push(PrintfToken::Literal("\x0B".to_string())),
                c => {
                    tokens.push(PrintfToken::Literal(format!("\\{c}")));
                }
            }
        } else if chars[i] == '%' {
            i += 1;
            if i >= chars.len() {
                tokens.push(PrintfToken::Literal("%".to_string()));
                break;
            }
            match chars[i] {
                '%' => tokens.push(PrintfToken::Percent),
                'p' => tokens.push(PrintfToken::Path),
                'f' => tokens.push(PrintfToken::Filename),
                'h' => tokens.push(PrintfToken::ParentDir),
                'd' => tokens.push(PrintfToken::Depth),
                's' => tokens.push(PrintfToken::Size),
                'k' => tokens.push(PrintfToken::SizeInBlocks),
                'b' => tokens.push(PrintfToken::BlockSize),
                'm' => tokens.push(PrintfToken::Permissions),
                'M' => tokens.push(PrintfToken::PermSymbolic),
                'n' => tokens.push(PrintfToken::Nlinks),
                'u' => tokens.push(PrintfToken::User),
                'g' => tokens.push(PrintfToken::Group),
                'U' => tokens.push(PrintfToken::Uid),
                'G' => tokens.push(PrintfToken::Gid),
                'i' => tokens.push(PrintfToken::Inode),
                'D' => tokens.push(PrintfToken::DeviceNumber),
                'y' => tokens.push(PrintfToken::Type),
                'l' => tokens.push(PrintfToken::LinkTarget),
                'P' => tokens.push(PrintfToken::SparseName),
                'H' => tokens.push(PrintfToken::StartingPoint),
                'a' => tokens.push(PrintfToken::TimeAccess),
                't' => tokens.push(PrintfToken::TimeModify),
                'c' => tokens.push(PrintfToken::TimeChange),
                'A' | 'T' | 'C' => {
                    let time_type = chars[i];
                    i += 1;
                    if i < chars.len() {
                        let spec = chars[i];
                        if spec == '@' {
                            match time_type {
                                'A' => tokens.push(PrintfToken::TimeAccessSecs),
                                'T' => tokens.push(PrintfToken::TimeModifySecs),
                                'C' => tokens.push(PrintfToken::TimeChangeSecs),
                                _ => unreachable!(),
                            }
                        } else {
                            let fmt_str = spec.to_string();
                            match time_type {
                                'A' => tokens.push(PrintfToken::TimeAccessFmt(fmt_str)),
                                'T' => tokens.push(PrintfToken::TimeModifyFmt(fmt_str)),
                                'C' => tokens.push(PrintfToken::TimeChangeFmt(fmt_str)),
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                c => {
                    tokens.push(PrintfToken::Literal(format!("%{c}")));
                }
            }
        } else {
            // collect literal string
            let start = i;
            while i < chars.len() && chars[i] != '%' && chars[i] != '\\' {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            tokens.push(PrintfToken::Literal(s));
            continue;
        }
        i += 1;
    }

    Ok(tokens)
}

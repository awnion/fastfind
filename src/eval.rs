use std::fs;
use std::io::Write;
use std::io::{self};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::expr::*;

/// Quit signal - set to true when -quit is evaluated
pub static QUIT_SIGNAL: AtomicBool = AtomicBool::new(false);

/// Context for expression evaluation
pub struct EvalContext<'a> {
    pub now: SystemTime,
    pub daystart: bool,
    pub daystart_time: SystemTime,
    pub starting_point: &'a Path,
    pub stdout: &'a mut dyn Write,
    pub depth: usize,
    pub batch_exec: Vec<(Vec<ExecArg>, Vec<std::path::PathBuf>, bool)>, /* (args, collected_paths, is_execdir) */
}

/// Execute all pending batch commands
pub fn flush_batch_exec(ctx: &mut EvalContext) {
    for (args, paths, is_execdir) in ctx.batch_exec.drain(..) {
        if paths.is_empty() {
            continue;
        }
        if is_execdir {
            // group by directory
            let mut by_dir: std::collections::HashMap<std::path::PathBuf, Vec<std::path::PathBuf>> =
                std::collections::HashMap::new();
            for p in paths {
                let dir = p.parent().unwrap_or(Path::new(".")).to_path_buf();
                let name = Path::new(".").join(p.file_name().unwrap_or_default());
                by_dir.entry(dir).or_default().push(name);
            }
            for (dir, names) in by_dir {
                exec_batch_command(&args, &names, Some(&dir));
            }
        } else {
            exec_batch_command(&args, &paths, None);
        }
    }
}

fn exec_batch_command(exec_args: &[ExecArg], paths: &[std::path::PathBuf], dir: Option<&Path>) {
    // Build command: replace {} with all paths
    let mut cmd_parts: Vec<String> = Vec::new();
    let mut placeholder_seen = false;

    for arg in exec_args {
        match arg {
            ExecArg::Literal(s) => cmd_parts.push(s.clone()),
            ExecArg::Placeholder => {
                placeholder_seen = true;
                for p in paths {
                    cmd_parts.push(p.to_str().unwrap_or("").to_string());
                }
            }
        }
    }

    if !placeholder_seen {
        // Append paths at end
        for p in paths {
            cmd_parts.push(p.to_str().unwrap_or("").to_string());
        }
    }

    if cmd_parts.is_empty() {
        return;
    }

    let program = cmd_parts.remove(0);
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_parts);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }

    match cmd.status() {
        Ok(_) => {}
        Err(e) => eprintln!("find: `{program}`: {e}"),
    }
}

/// File entry information for evaluation
pub struct EntryInfo<'a> {
    pub path: &'a Path,
    pub depth: usize,
    pub file_type: EntryType,
    pub metadata: Option<fs::Metadata>,
    pub is_dir: bool,
    pub should_prune: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    Pipe,
    Socket,
    Unknown,
}

impl EntryType {
    #[inline]
    pub fn matches(self, ft: FileType) -> bool {
        matches!(
            (self, ft),
            (EntryType::File, FileType::File)
                | (EntryType::Directory, FileType::Directory)
                | (EntryType::Symlink, FileType::Symlink)
                | (EntryType::BlockDevice, FileType::BlockDevice)
                | (EntryType::CharDevice, FileType::CharDevice)
                | (EntryType::Pipe, FileType::Pipe)
                | (EntryType::Socket, FileType::Socket)
        )
    }

    pub fn to_char(self) -> char {
        match self {
            EntryType::File => 'f',
            EntryType::Directory => 'd',
            EntryType::Symlink => 'l',
            EntryType::BlockDevice => 'b',
            EntryType::CharDevice => 'c',
            EntryType::Pipe => 'p',
            EntryType::Socket => 's',
            EntryType::Unknown => 'U',
        }
    }

    #[cfg(unix)]
    pub fn from_mode(mode: u32) -> Self {
        #[allow(clippy::unnecessary_cast)]
        match mode & libc::S_IFMT as u32 {
            m if m == libc::S_IFREG as u32 => EntryType::File,
            m if m == libc::S_IFDIR as u32 => EntryType::Directory,
            m if m == libc::S_IFLNK as u32 => EntryType::Symlink,
            m if m == libc::S_IFBLK as u32 => EntryType::BlockDevice,
            m if m == libc::S_IFCHR as u32 => EntryType::CharDevice,
            m if m == libc::S_IFIFO as u32 => EntryType::Pipe,
            m if m == libc::S_IFSOCK as u32 => EntryType::Socket,
            _ => EntryType::Unknown,
        }
    }
}

impl<'a> EntryInfo<'a> {
    pub fn get_metadata(&mut self) -> io::Result<&fs::Metadata> {
        if self.metadata.is_none() {
            self.metadata = Some(fs::symlink_metadata(self.path)?);
        }
        Ok(self.metadata.as_ref().unwrap())
    }

    pub fn get_follow_metadata(&mut self) -> io::Result<fs::Metadata> {
        fs::metadata(self.path)
    }
}

/// Evaluate an expression against a file entry
#[inline]
pub fn evaluate(expr: &Expr, entry: &mut EntryInfo, ctx: &mut EvalContext) -> io::Result<bool> {
    if QUIT_SIGNAL.load(Ordering::Relaxed) {
        return Ok(false);
    }

    match expr {
        // Operators
        Expr::And(exprs) => {
            for e in exprs {
                if !evaluate(e, entry, ctx)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Expr::Or(exprs) => {
            for e in exprs {
                if evaluate(e, entry, ctx)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Expr::Not(e) => Ok(!evaluate(e, entry, ctx)?),
        Expr::List(exprs) => {
            let mut result = true;
            for e in exprs {
                result = evaluate(e, entry, ctx)?;
            }
            Ok(result)
        }

        // Tests
        Expr::True => Ok(true),
        Expr::False => Ok(false),

        Expr::Name(pat) | Expr::IName(pat) => {
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let name_bytes = entry.path.file_name().unwrap_or_default().as_bytes();
                Ok(pat.is_match_bytes(name_bytes))
            }
            #[cfg(not(unix))]
            {
                let name = entry.path.file_name().unwrap_or_default().to_str().unwrap_or("");
                Ok(pat.is_match(name))
            }
        }
        Expr::Path(pat) | Expr::IPath(pat) => {
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let path_bytes = entry.path.as_os_str().as_bytes();
                Ok(pat.is_match_bytes(path_bytes))
            }
            #[cfg(not(unix))]
            {
                let path_str = entry.path.to_str().unwrap_or("");
                Ok(pat.is_match(path_str))
            }
        }
        Expr::LName(pat) => {
            if !matches!(entry.file_type, EntryType::Symlink) {
                return Ok(false);
            }
            match fs::read_link(entry.path) {
                Ok(target) => {
                    let target_str = target.to_str().unwrap_or("");
                    Ok(pat.is_match(target_str))
                }
                Err(_) => Ok(false),
            }
        }
        Expr::ILName(pat) => {
            if !matches!(entry.file_type, EntryType::Symlink) {
                return Ok(false);
            }
            match fs::read_link(entry.path) {
                Ok(target) => {
                    let target_str = target.to_str().unwrap_or("");
                    Ok(pat.is_match(target_str))
                }
                Err(_) => Ok(false),
            }
        }
        Expr::Regex(re) => {
            let path_str = entry.path.to_str().unwrap_or("");
            Ok(re.is_match(path_str))
        }
        Expr::IRegex(re) => {
            let path_str = entry.path.to_str().unwrap_or("");
            Ok(re.is_match(path_str))
        }

        Expr::Type(types) => Ok(types.iter().any(|t| entry.file_type.matches(*t))),
        Expr::XType(types) => {
            // -xtype checks the type of the file the symlink points to
            if matches!(entry.file_type, EntryType::Symlink) {
                match entry.get_follow_metadata() {
                    Ok(meta) => {
                        let ft = file_type_from_metadata(&meta);
                        Ok(types.iter().any(|t| ft.matches(*t)))
                    }
                    Err(_) => Ok(false),
                }
            } else {
                Ok(types.iter().any(|t| entry.file_type.matches(*t)))
            }
        }

        Expr::Size { cmp, size, unit } => {
            let meta = entry.get_metadata()?;
            let file_size = meta.len();
            let unit_bytes = unit.bytes();
            // GNU find rounds up for non-byte units
            let in_units = if matches!(unit, SizeUnit::Bytes) {
                file_size
            } else {
                file_size.div_ceil(unit_bytes)
            };
            Ok(cmp.matches_u64(in_units, *size))
        }

        Expr::Empty => {
            if entry.is_dir {
                // Check if directory is empty
                match fs::read_dir(entry.path) {
                    Ok(mut entries) => Ok(entries.next().is_none()),
                    Err(_) => Ok(false),
                }
            } else {
                let meta = entry.get_metadata()?;
                Ok(meta.len() == 0)
            }
        }

        #[cfg(unix)]
        Expr::Perm { mode, match_type } => {
            let meta = entry.get_metadata()?;
            let file_mode = meta.mode() & 0o7777;
            match match_type {
                PermMatch::Exact => Ok(file_mode == *mode),
                PermMatch::All => Ok(file_mode & mode == *mode),
                PermMatch::Any => {
                    if *mode == 0 {
                        Ok(true)
                    } else {
                        Ok(file_mode & mode != 0)
                    }
                }
            }
        }
        #[cfg(not(unix))]
        Expr::Perm { .. } => Ok(false),

        Expr::Readable => Ok(test_access(entry.path, libc::R_OK)),
        Expr::Writable => Ok(test_access(entry.path, libc::W_OK)),
        Expr::Executable => Ok(test_access(entry.path, libc::X_OK)),

        #[cfg(unix)]
        Expr::User(name) => {
            let meta = entry.get_metadata()?;
            let uid = meta.uid();
            // Try to parse as numeric uid first
            if let Ok(target_uid) = name.parse::<u32>() {
                return Ok(uid == target_uid);
            }
            match lookup_uid_by_name(name) {
                Some(target_uid) => Ok(uid == target_uid),
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("-user: `{name}` is not the name of a known user"),
                )),
            }
        }
        #[cfg(not(unix))]
        Expr::User(_) => Ok(false),

        #[cfg(unix)]
        Expr::Group(name) => {
            let meta = entry.get_metadata()?;
            let gid = meta.gid();
            if let Ok(target_gid) = name.parse::<u32>() {
                return Ok(gid == target_gid);
            }
            match lookup_gid_by_name(name) {
                Some(target_gid) => Ok(gid == target_gid),
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("-group: `{name}` is not the name of a known group"),
                )),
            }
        }
        #[cfg(not(unix))]
        Expr::Group(_) => Ok(false),

        #[cfg(unix)]
        Expr::Uid(cmp, target) => {
            let meta = entry.get_metadata()?;
            Ok(cmp.matches_u64(meta.uid() as u64, *target))
        }
        #[cfg(not(unix))]
        Expr::Uid(_, _) => Ok(false),

        #[cfg(unix)]
        Expr::Gid(cmp, target) => {
            let meta = entry.get_metadata()?;
            Ok(cmp.matches_u64(meta.gid() as u64, *target))
        }
        #[cfg(not(unix))]
        Expr::Gid(_, _) => Ok(false),

        #[cfg(unix)]
        Expr::NoUser => {
            let meta = entry.get_metadata()?;
            Ok(lookup_name_by_uid(meta.uid()).is_none())
        }
        #[cfg(not(unix))]
        Expr::NoUser => Ok(false),

        #[cfg(unix)]
        Expr::NoGroup => {
            let meta = entry.get_metadata()?;
            Ok(lookup_name_by_gid(meta.gid()).is_none())
        }
        #[cfg(not(unix))]
        Expr::NoGroup => Ok(false),

        #[cfg(unix)]
        Expr::Links(cmp, target) => {
            let meta = entry.get_metadata()?;
            Ok(cmp.matches_u64(meta.nlink(), *target))
        }
        #[cfg(not(unix))]
        Expr::Links(_, _) => Ok(false),

        #[cfg(unix)]
        Expr::Inum(cmp, target) => {
            let meta = entry.get_metadata()?;
            Ok(cmp.matches_u64(meta.ino(), *target))
        }
        #[cfg(not(unix))]
        Expr::Inum(_, _) => Ok(false),

        #[cfg(unix)]
        Expr::SameFile { dev, ino } => {
            let meta = entry.get_metadata()?;
            Ok(meta.dev() == *dev && meta.ino() == *ino)
        }
        #[cfg(not(unix))]
        Expr::SameFile { .. } => Ok(false),

        // Time tests
        Expr::MTime(cmp, n) => {
            let meta = entry.get_metadata()?;
            let mtime = meta.modified()?;
            Ok(check_time_days(mtime, *cmp, *n, ctx))
        }
        Expr::MMin(cmp, n) => {
            let meta = entry.get_metadata()?;
            let mtime = meta.modified()?;
            Ok(check_time_mins(mtime, *cmp, *n, ctx))
        }
        Expr::ATime(cmp, n) => {
            let meta = entry.get_metadata()?;
            let atime = meta.accessed()?;
            Ok(check_time_days(atime, *cmp, *n, ctx))
        }
        Expr::AMin(cmp, n) => {
            let meta = entry.get_metadata()?;
            let atime = meta.accessed()?;
            Ok(check_time_mins(atime, *cmp, *n, ctx))
        }
        #[cfg(unix)]
        Expr::CTime(cmp, n) => {
            let meta = entry.get_metadata()?;
            let ctime = UNIX_EPOCH + Duration::from_secs(meta.ctime() as u64);
            Ok(check_time_days(ctime, *cmp, *n, ctx))
        }
        #[cfg(not(unix))]
        Expr::CTime(cmp, n) => {
            let meta = entry.get_metadata()?;
            let mtime = meta.modified()?;
            Ok(check_time_days(mtime, *cmp, *n, ctx))
        }
        #[cfg(unix)]
        Expr::CMin(cmp, n) => {
            let meta = entry.get_metadata()?;
            let ctime = UNIX_EPOCH + Duration::from_secs(meta.ctime() as u64);
            Ok(check_time_mins(ctime, *cmp, *n, ctx))
        }
        #[cfg(not(unix))]
        Expr::CMin(cmp, n) => {
            let meta = entry.get_metadata()?;
            let mtime = meta.modified()?;
            Ok(check_time_mins(mtime, *cmp, *n, ctx))
        }

        Expr::Newer(ref_time) => {
            let meta = entry.get_metadata()?;
            let mtime = meta.modified()?;
            Ok(mtime > *ref_time)
        }
        Expr::ANewerM(ref_time) => {
            let meta = entry.get_metadata()?;
            let atime = meta.accessed()?;
            Ok(atime > *ref_time)
        }
        #[cfg(unix)]
        Expr::CNewerM(ref_time) => {
            let meta = entry.get_metadata()?;
            let ctime = UNIX_EPOCH + Duration::from_secs(meta.ctime() as u64);
            Ok(ctime > *ref_time)
        }
        #[cfg(not(unix))]
        Expr::CNewerM(ref_time) => {
            let meta = entry.get_metadata()?;
            let mtime = meta.modified()?;
            Ok(mtime > *ref_time)
        }

        Expr::NewerXY { x, reference, .. } => {
            let meta = entry.get_metadata()?;
            let entry_time = get_time_ref(meta, *x)?;
            Ok(entry_time > *reference)
        }

        #[cfg(unix)]
        Expr::Used(cmp, n) => {
            let meta = entry.get_metadata()?;
            let atime = meta.atime();
            let ctime = meta.ctime();
            let days = (atime - ctime) / 86400;
            Ok(cmp.matches(days, *n))
        }
        #[cfg(not(unix))]
        Expr::Used(_, _) => Ok(false),

        #[cfg(unix)]
        Expr::FsType(expected) => check_fstype(entry.path, expected),

        // Actions
        Expr::Print => {
            write_path(ctx.stdout, entry.path)?;
            ctx.stdout.write_all(b"\n")?;
            Ok(true)
        }
        Expr::Print0 => {
            write_path(ctx.stdout, entry.path)?;
            ctx.stdout.write_all(b"\0")?;
            Ok(true)
        }
        Expr::Printf(tokens) => {
            eval_printf(tokens, entry, ctx)?;
            Ok(true)
        }
        Expr::Ls => {
            eval_ls(entry, ctx.stdout)?;
            Ok(true)
        }
        Expr::FPrint(path) => {
            let mut f = open_append(path)?;
            write_path(&mut f, entry.path)?;
            f.write_all(b"\n")?;
            Ok(true)
        }
        Expr::FPrint0(path) => {
            let mut f = open_append(path)?;
            write_path(&mut f, entry.path)?;
            f.write_all(b"\0")?;
            Ok(true)
        }
        Expr::FPrintf(path, tokens) => {
            let mut f = open_append(path)?;
            eval_printf_to(tokens, entry, ctx, &mut f)?;
            Ok(true)
        }
        Expr::FLs(path) => {
            let mut f = open_append(path)?;
            eval_ls(entry, &mut f)?;
            Ok(true)
        }

        Expr::Exec { args, batch } => {
            if *batch {
                let path_buf = entry.path.to_path_buf();
                // Always push to the first non-execdir batch, or create one
                if let Some(entry) = ctx.batch_exec.iter_mut().find(|(_, _, d)| !d) {
                    entry.1.push(path_buf);
                } else {
                    ctx.batch_exec.push((args.clone(), vec![path_buf], false));
                }
                Ok(true)
            } else {
                exec_command(args, entry.path, None)
            }
        }
        Expr::ExecDir { args, batch } => {
            if *batch {
                let path_buf = entry.path.to_path_buf();
                if let Some(entry) = ctx.batch_exec.iter_mut().find(|(_, _, d)| *d) {
                    entry.1.push(path_buf);
                } else {
                    ctx.batch_exec.push((args.clone(), vec![path_buf], true));
                }
                Ok(true)
            } else {
                let dir = entry.path.parent();
                let name = entry.path.file_name().map(|n| Path::new(".").join(n));
                let name_ref = name.as_deref().unwrap_or(entry.path);
                exec_command(args, name_ref, dir)
            }
        }
        Expr::Ok { args } => ok_command(args, entry.path, None),
        Expr::OkDir { args } => {
            let dir = entry.path.parent();
            let name = entry.path.file_name().map(|n| Path::new(".").join(n));
            let name_ref = name.as_deref().unwrap_or(entry.path);
            ok_command(args, name_ref, dir)
        }

        Expr::Delete => {
            if entry.is_dir {
                match fs::remove_dir(entry.path) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("find: cannot delete `{}`: {e}", entry.path.display());
                        Ok(false)
                    }
                }
            } else {
                match fs::remove_file(entry.path) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("find: cannot delete `{}`: {e}", entry.path.display());
                        Ok(false)
                    }
                }
            }
        }

        Expr::Prune => {
            entry.should_prune = true;
            Ok(true)
        }

        Expr::Quit => {
            QUIT_SIGNAL.store(true, Ordering::Relaxed);
            Ok(true)
        }
    }
}

#[inline]
fn check_time_days(file_time: SystemTime, cmp: Cmp, n: i64, ctx: &EvalContext) -> bool {
    let reference = if ctx.daystart { ctx.daystart_time } else { ctx.now };
    let diff = reference.duration_since(file_time).unwrap_or(Duration::ZERO);
    // GNU find semantics: file age in fractional days, rounded down
    // -mtime 0: 0 <= age < 1 day
    // -mtime 1: 1 <= age < 2 days
    // -mtime +1: age >= 2 days (i.e., floor(age) > 1)
    // -mtime -1: age < 1 day (i.e., floor(age) < 1)
    let days = (diff.as_secs() / 86400) as i64;
    cmp.matches(days, n)
}

#[inline]
fn check_time_mins(file_time: SystemTime, cmp: Cmp, n: i64, ctx: &EvalContext) -> bool {
    let reference = if ctx.daystart { ctx.daystart_time } else { ctx.now };
    let diff = reference.duration_since(file_time).unwrap_or(Duration::ZERO);
    let mins = (diff.as_secs() / 60) as i64;
    cmp.matches(mins, n)
}

#[cfg(unix)]
fn get_time_ref(meta: &fs::Metadata, time_ref: TimeRef) -> io::Result<SystemTime> {
    match time_ref {
        TimeRef::Modify => meta.modified(),
        TimeRef::Access => meta.accessed(),
        TimeRef::Change => Ok(UNIX_EPOCH + Duration::from_secs(meta.ctime() as u64)),
        TimeRef::Birth => meta.created(),
    }
}

#[cfg(not(unix))]
fn get_time_ref(meta: &fs::Metadata, time_ref: TimeRef) -> io::Result<SystemTime> {
    match time_ref {
        TimeRef::Modify | TimeRef::Change => meta.modified(),
        TimeRef::Access => meta.accessed(),
        TimeRef::Birth => meta.created(),
    }
}

#[cfg(unix)]
#[inline]
fn write_path(out: &mut dyn Write, path: &Path) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    out.write_all(path.as_os_str().as_bytes())
}

#[cfg(not(unix))]
fn write_path(out: &mut dyn Write, path: &Path) -> io::Result<()> {
    write!(out, "{}", path.display())
}

fn file_type_from_metadata(meta: &fs::Metadata) -> EntryType {
    let ft = meta.file_type();
    if ft.is_file() {
        EntryType::File
    } else if ft.is_dir() {
        EntryType::Directory
    } else if ft.is_symlink() {
        EntryType::Symlink
    } else {
        #[cfg(unix)]
        {
            EntryType::from_mode(meta.mode())
        }
        #[cfg(not(unix))]
        {
            EntryType::Unknown
        }
    }
}

fn exec_command(exec_args: &[ExecArg], path: &Path, dir: Option<&Path>) -> io::Result<bool> {
    let path_str = path.to_str().unwrap_or("");
    let mut cmd_parts: Vec<String> = exec_args
        .iter()
        .map(|a| match a {
            ExecArg::Literal(s) => s.clone(),
            ExecArg::Placeholder => path_str.to_string(),
        })
        .collect();

    if cmd_parts.is_empty() {
        return Ok(false);
    }

    let program = cmd_parts.remove(0);
    let mut cmd = Command::new(&program);
    cmd.args(&cmd_parts);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }

    match cmd.status() {
        Ok(status) => Ok(status.success()),
        Err(e) => {
            eprintln!("find: `{program}`: {e}");
            Ok(false)
        }
    }
}

fn ok_command(exec_args: &[ExecArg], path: &Path, dir: Option<&Path>) -> io::Result<bool> {
    let path_str = path.to_str().unwrap_or("");
    let cmd_str: Vec<String> = exec_args
        .iter()
        .map(|a| match a {
            ExecArg::Literal(s) => s.clone(),
            ExecArg::Placeholder => path_str.to_string(),
        })
        .collect();

    eprint!("< {} ... {} > ? ", cmd_str.join(" "), path.display());
    let _ = io::stderr().flush();

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    let response = response.trim().to_lowercase();

    if response == "y" || response == "yes" {
        exec_command(exec_args, path, dir)
    } else {
        Ok(false)
    }
}

#[cfg(unix)]
fn test_access(path: &Path, mode: libc::c_int) -> bool {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = match CString::new(path.as_os_str().as_bytes()) {
        Ok(p) => p,
        Err(_) => return false,
    };
    unsafe { libc::access(c_path.as_ptr(), mode) == 0 }
}

#[cfg(not(unix))]
fn test_access(_path: &Path, _mode: i32) -> bool {
    false
}

#[cfg(unix)]
fn lookup_uid_by_name(name: &str) -> Option<u32> {
    use std::ffi::CString;
    let c_name = CString::new(name).ok()?;
    unsafe {
        let pw = libc::getpwnam(c_name.as_ptr());
        if pw.is_null() { None } else { Some((*pw).pw_uid) }
    }
}

#[cfg(unix)]
fn lookup_gid_by_name(name: &str) -> Option<u32> {
    use std::ffi::CString;
    let c_name = CString::new(name).ok()?;
    unsafe {
        let gr = libc::getgrnam(c_name.as_ptr());
        if gr.is_null() { None } else { Some((*gr).gr_gid) }
    }
}

#[cfg(unix)]
fn lookup_name_by_uid(uid: u32) -> Option<String> {
    use std::ffi::CStr;
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            None
        } else {
            CStr::from_ptr((*pw).pw_name).to_str().ok().map(|s| s.to_string())
        }
    }
}

#[cfg(unix)]
fn lookup_name_by_gid(gid: u32) -> Option<String> {
    use std::ffi::CStr;
    unsafe {
        let gr = libc::getgrgid(gid);
        if gr.is_null() {
            None
        } else {
            CStr::from_ptr((*gr).gr_name).to_str().ok().map(|s| s.to_string())
        }
    }
}

fn open_append(path: &Path) -> io::Result<fs::File> {
    fs::OpenOptions::new().create(true).append(true).open(path)
}

// -ls format: inode blocks permissions nlinks user group size date name
fn eval_ls(entry: &mut EntryInfo, out: &mut dyn Write) -> io::Result<()> {
    #[cfg(unix)]
    {
        let meta = entry.get_metadata()?;
        let ino = meta.ino();
        let blocks = meta.blocks();
        let mode = meta.mode();
        let nlink = meta.nlink();
        let uid = meta.uid();
        let gid = meta.gid();
        let size = meta.len();

        let user = lookup_name_by_uid(uid).unwrap_or_else(|| uid.to_string());
        let group = lookup_name_by_gid(gid).unwrap_or_else(|| gid.to_string());

        let mtime_secs = meta.mtime();
        let date_str = format_time_ls(mtime_secs);
        let perm_str = format_mode(mode);

        write!(
            out,
            "{:>7} {:>4} {} {:>3} {:<8} {:<8} {:>8} {} {}",
            ino,
            blocks / 2, // ls uses 1K blocks
            perm_str,
            nlink,
            user,
            group,
            size,
            date_str,
            entry.path.display()
        )?;

        // If symlink, show target on the same line
        if matches!(entry.file_type, EntryType::Symlink)
            && let Ok(target) = fs::read_link(entry.path)
        {
            write!(out, " -> {}", target.display())?;
        }
        writeln!(out)?;
    }
    #[cfg(not(unix))]
    {
        writeln!(out, "{}", entry.path.display())?;
    }
    Ok(())
}

#[cfg(unix)]
fn format_mode(mode: u32) -> String {
    #[allow(clippy::unnecessary_cast)]
    let ft = match mode & libc::S_IFMT as u32 {
        m if m == libc::S_IFDIR as u32 => 'd',
        m if m == libc::S_IFLNK as u32 => 'l',
        m if m == libc::S_IFBLK as u32 => 'b',
        m if m == libc::S_IFCHR as u32 => 'c',
        m if m == libc::S_IFIFO as u32 => 'p',
        m if m == libc::S_IFSOCK as u32 => 's',
        _ => '-',
    };

    let mut s = String::with_capacity(10);
    s.push(ft);
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    #[allow(clippy::unnecessary_cast)]
    s.push(if mode & libc::S_ISUID as u32 != 0 {
        if mode & 0o100 != 0 { 's' } else { 'S' }
    } else if mode & 0o100 != 0 {
        'x'
    } else {
        '-'
    });
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    #[allow(clippy::unnecessary_cast)]
    s.push(if mode & libc::S_ISGID as u32 != 0 {
        if mode & 0o010 != 0 { 's' } else { 'S' }
    } else if mode & 0o010 != 0 {
        'x'
    } else {
        '-'
    });
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    #[allow(clippy::unnecessary_cast)]
    s.push(if mode & libc::S_ISVTX as u32 != 0 {
        if mode & 0o001 != 0 { 't' } else { 'T' }
    } else if mode & 0o001 != 0 {
        'x'
    } else {
        '-'
    });
    s
}

#[cfg(unix)]
fn format_time_ls(secs: i64) -> String {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    let six_months = 180 * 86400;
    let datetime: chrono_format::DateTime = chrono_format::from_timestamp(secs);

    if (now - secs).abs() > six_months {
        format!("{} {:>2}  {}", datetime.month_abbr, datetime.day, datetime.year)
    } else {
        format!(
            "{} {:>2} {:02}:{:02}",
            datetime.month_abbr, datetime.day, datetime.hour, datetime.minute
        )
    }
}

// Simple date formatting without external deps
mod chrono_format {
    pub struct DateTime {
        pub year: i32,
        pub month_abbr: &'static str,
        pub day: u32,
        pub hour: u32,
        pub minute: u32,
        pub second: u32,
    }

    const MONTHS: [&str; 12] =
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    pub fn from_timestamp(secs: i64) -> DateTime {
        // Convert Unix timestamp to date components
        let days = (secs / 86400) as i32;
        let time_of_day = ((secs % 86400) + 86400) as u32 % 86400;

        let hour = time_of_day / 3600;
        let minute = (time_of_day % 3600) / 60;
        let second = time_of_day % 60;

        // Days since epoch (1970-01-01)
        let (year, month, day) = days_to_ymd(days + 719468); // shift to year 0

        DateTime { year, month_abbr: MONTHS[month as usize], day: day as u32, hour, minute, second }
    }

    fn days_to_ymd(days: i32) -> (i32, i32, i32) {
        // Civil days to y/m/d using Howard Hinnant's algorithm
        let era = if days >= 0 { days } else { days - 146096 } / 146097;
        let doe = days - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        (y, m - 1, d) // month is 0-indexed for MONTHS array
    }
}

fn eval_printf(
    tokens: &[PrintfToken],
    entry: &mut EntryInfo,
    ctx: &mut EvalContext,
) -> io::Result<()> {
    // Temporarily take stdout to avoid double borrow
    let stdout_ptr = ctx.stdout as *mut dyn Write;
    // SAFETY: we're not using ctx.stdout while out is alive, and eval_printf_to
    // only uses ctx for non-stdout fields
    let out = unsafe { &mut *stdout_ptr };
    eval_printf_to(tokens, entry, ctx, out)
}

fn eval_printf_to(
    tokens: &[PrintfToken],
    entry: &mut EntryInfo,
    ctx: &mut EvalContext,
    out: &mut dyn Write,
) -> io::Result<()> {
    for token in tokens {
        match token {
            PrintfToken::Literal(s) => write!(out, "{s}")?,
            PrintfToken::Percent => write!(out, "%")?,
            PrintfToken::Path => write!(out, "{}", entry.path.display())?,
            PrintfToken::Filename => {
                let name = entry.path.file_name().unwrap_or(entry.path.as_os_str());
                write!(out, "{}", name.to_str().unwrap_or(""))?;
            }
            PrintfToken::ParentDir => {
                let parent = entry.path.parent().unwrap_or(entry.path);
                write!(out, "{}", parent.display())?;
            }
            PrintfToken::Depth => write!(out, "{}", entry.depth)?,
            PrintfToken::Size => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.len())?;
                } else {
                    write!(out, "0")?;
                }
            }
            #[cfg(unix)]
            PrintfToken::SizeInBlocks => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.blocks() / 2)?; // 1K blocks
                } else {
                    write!(out, "0")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::SizeInBlocks => write!(out, "0")?,
            #[cfg(unix)]
            PrintfToken::BlockSize => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.blocks())?;
                } else {
                    write!(out, "0")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::BlockSize => write!(out, "0")?,
            #[cfg(unix)]
            PrintfToken::Permissions => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{:04o}", meta.mode() & 0o7777)?;
                } else {
                    write!(out, "0000")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::Permissions => write!(out, "0000")?,
            #[cfg(unix)]
            PrintfToken::PermSymbolic => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", format_mode(meta.mode()))?;
                } else {
                    write!(out, "----------")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::PermSymbolic => write!(out, "----------")?,
            #[cfg(unix)]
            PrintfToken::Nlinks => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.nlink())?;
                } else {
                    write!(out, "0")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::Nlinks => write!(out, "0")?,
            #[cfg(unix)]
            PrintfToken::User => {
                if let Ok(meta) = entry.get_metadata() {
                    let name =
                        lookup_name_by_uid(meta.uid()).unwrap_or_else(|| meta.uid().to_string());
                    write!(out, "{name}")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::User => {}
            #[cfg(unix)]
            PrintfToken::Group => {
                if let Ok(meta) = entry.get_metadata() {
                    let name =
                        lookup_name_by_gid(meta.gid()).unwrap_or_else(|| meta.gid().to_string());
                    write!(out, "{name}")?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::Group => {}
            #[cfg(unix)]
            PrintfToken::Uid => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.uid())?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::Uid => write!(out, "0")?,
            #[cfg(unix)]
            PrintfToken::Gid => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.gid())?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::Gid => write!(out, "0")?,
            #[cfg(unix)]
            PrintfToken::Inode => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.ino())?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::Inode => write!(out, "0")?,
            #[cfg(unix)]
            PrintfToken::DeviceNumber => {
                if let Ok(meta) = entry.get_metadata() {
                    write!(out, "{}", meta.dev())?;
                }
            }
            #[cfg(not(unix))]
            PrintfToken::DeviceNumber => write!(out, "0")?,
            PrintfToken::Type => {
                write!(out, "{}", entry.file_type.to_char())?;
            }
            PrintfToken::LinkTarget => {
                if matches!(entry.file_type, EntryType::Symlink)
                    && let Ok(target) = fs::read_link(entry.path)
                {
                    write!(out, "{}", target.display())?;
                }
            }
            PrintfToken::Newline => writeln!(out)?,
            PrintfToken::Tab => write!(out, "\t")?,
            PrintfToken::NullChar => out.write_all(b"\0")?,
            PrintfToken::Backslash => write!(out, "\\")?,
            PrintfToken::TimeAccess | PrintfToken::TimeModify | PrintfToken::TimeChange => {
                #[cfg(unix)]
                {
                    let meta = entry.get_metadata()?;
                    let secs = match token {
                        PrintfToken::TimeAccess => meta.atime(),
                        PrintfToken::TimeModify => meta.mtime(),
                        PrintfToken::TimeChange => meta.ctime(),
                        _ => unreachable!(),
                    };
                    let dt = chrono_format::from_timestamp(secs);
                    write!(
                        out,
                        "{} {:>2} {:02}:{:02}:{:02}.0000000000 {}",
                        dt.month_abbr, dt.day, dt.hour, dt.minute, dt.second, dt.year,
                    )?;
                }
            }
            PrintfToken::TimeAccessSecs => {
                #[cfg(unix)]
                {
                    let meta = entry.get_metadata()?;
                    write!(out, "{}", meta.atime())?;
                }
            }
            PrintfToken::TimeModifySecs => {
                #[cfg(unix)]
                {
                    let meta = entry.get_metadata()?;
                    write!(out, "{}", meta.mtime())?;
                }
            }
            PrintfToken::TimeChangeSecs => {
                #[cfg(unix)]
                {
                    let meta = entry.get_metadata()?;
                    write!(out, "{}", meta.ctime())?;
                }
            }
            PrintfToken::TimeAccessFmt(_fmt)
            | PrintfToken::TimeModifyFmt(_fmt)
            | PrintfToken::TimeChangeFmt(_fmt) => {
                // Basic strftime-like format - just print timestamp for now
                #[cfg(unix)]
                {
                    let meta = entry.get_metadata()?;
                    let secs = match token {
                        PrintfToken::TimeAccessFmt(_) => meta.atime(),
                        PrintfToken::TimeModifyFmt(_) => meta.mtime(),
                        PrintfToken::TimeChangeFmt(_) => meta.ctime(),
                        _ => unreachable!(),
                    };
                    write!(out, "{secs}")?;
                }
            }
            PrintfToken::SparseName => {
                // %P - path relative to starting point
                if let Ok(rel) = entry.path.strip_prefix(ctx.starting_point) {
                    write!(out, "{}", rel.display())?;
                } else {
                    write!(out, "{}", entry.path.display())?;
                }
            }
            PrintfToken::StartingPoint => {
                write!(out, "{}", ctx.starting_point.display())?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn check_fstype(path: &Path, expected: &str) -> io::Result<bool> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    unsafe {
        let mut buf: libc::statfs = std::mem::zeroed();
        if libc::statfs(c_path.as_ptr(), &mut buf) != 0 {
            return Err(io::Error::last_os_error());
        }

        #[cfg(target_os = "macos")]
        {
            let fstype = std::ffi::CStr::from_ptr(buf.f_fstypename.as_ptr()).to_str().unwrap_or("");
            Ok(fstype == expected)
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, f_type is a numeric value
            let fstype_name = linux_fstype_name(buf.f_type);
            Ok(fstype_name == expected)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let _ = buf;
            let _ = expected;
            Ok(false)
        }
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn linux_fstype_name(f_type: libc::__fsword_t) -> &'static str {
    match f_type {
        0x9123683E => "btrfs",
        0xEF53 => "ext2/ext3/ext4",
        0x6969 => "nfs",
        0x01021994 => "tmpfs",
        0x58465342 => "xfs",
        0x2FC12FC1 => "zfs",
        0x9FA0 => "proc",
        0x62656572 => "sysfs",
        0x64626720 => "debugfs",
        0x1CD1 => "devpts",
        0x794C7630 => "overlayfs",
        0x61756673 => "aufs",
        0xCAFE001 => "cifs",
        0xFF534D42 => "cifs",
        _ => "unknown",
    }
}

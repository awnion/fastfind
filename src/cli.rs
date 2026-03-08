use clap::Arg;
use clap::Command;
use clap::builder::styling;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_SHA"), ")");

pub fn version_string() -> String {
    format!("find (fastfind) {VERSION}\nhttps://crates.io/crates/fastfind")
}

pub fn build_cli() -> Command {
    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::Yellow.on_default().bold())
        .usage(styling::AnsiColor::Yellow.on_default().bold())
        .literal(styling::AnsiColor::Green.on_default().bold())
        .placeholder(styling::AnsiColor::Cyan.on_default());

    Command::new("find")
        .version(VERSION)
        .about("Fast, parallel, GNU find replacement optimized for AI agents and large codebases\nhttps://crates.io/crates/fastfind")
        .override_usage("find [-H|-L|-P] [path...] [expression]")
        .styles(styles)
        // GNU find uses single-dash `-help`/`-version` which clap doesn't support,
        // so we handle them in our parser and use clap only for help rendering.
        .disable_help_flag(true)
        .disable_version_flag(true)
        .args(symlink_args())
        .args(global_args())
        .args(test_args())
        .args(action_args())
        .args(operator_args())
        .after_help(AFTER_HELP)
}

fn symlink_args() -> Vec<Arg> {
    vec![
        Arg::new("H")
            .short('H')
            .help("Don't follow symlinks (except on command line args)")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Symlink options"),
        Arg::new("L")
            .short('L')
            .help("Follow all symlinks")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Symlink options"),
        Arg::new("P")
            .short('P')
            .help("Never follow symlinks (default)")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Symlink options"),
    ]
}

fn global_args() -> Vec<Arg> {
    vec![
        Arg::new("maxdepth")
            .long("maxdepth")
            .value_name("N")
            .help("Descend at most N levels")
            .help_heading("Global options"),
        Arg::new("mindepth")
            .long("mindepth")
            .value_name("N")
            .help("Skip entries at depth less than N")
            .help_heading("Global options"),
        Arg::new("depth")
            .long("depth")
            .help("Process directory contents before the directory itself")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Global options"),
        Arg::new("xdev")
            .long("xdev")
            .help("Don't descend into other filesystems")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Global options"),
        Arg::new("noleaf")
            .long("noleaf")
            .help("Don't optimize directory link count (no-op, for compatibility)")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Global options"),
    ]
}

fn test_args() -> Vec<Arg> {
    vec![
        Arg::new("name")
            .long("name")
            .value_name("PATTERN")
            .help("Match filename against shell glob"),
        Arg::new("iname").long("iname").value_name("PATTERN").help("Case-insensitive -name"),
        Arg::new("path")
            .long("path")
            .value_name("PATTERN")
            .help("Match full path against shell glob"),
        Arg::new("ipath").long("ipath").value_name("PATTERN").help("Case-insensitive -path"),
        Arg::new("regex").long("regex").value_name("PATTERN").help("Match full path against regex"),
        Arg::new("iregex").long("iregex").value_name("PATTERN").help("Case-insensitive -regex"),
        Arg::new("type")
            .long("type")
            .value_name("TYPE")
            .help("File type: f,d,l,b,c,p,s (comma-separated)"),
        Arg::new("xtype")
            .long("xtype")
            .value_name("TYPE")
            .help("Like -type but checks symlink target"),
        Arg::new("size")
            .long("size")
            .value_name("N[ckMG]")
            .help("File size (+N: greater, -N: less, N: exact)"),
        Arg::new("empty")
            .long("empty")
            .help("Empty file or directory")
            .action(clap::ArgAction::SetTrue),
        Arg::new("perm")
            .long("perm")
            .value_name("MODE")
            .help("Permission bits (-MODE: all, /MODE: any, MODE: exact)"),
        Arg::new("user").long("user").value_name("NAME").help("File owner by name or uid"),
        Arg::new("group").long("group").value_name("NAME").help("File group by name or gid"),
        Arg::new("nouser")
            .long("nouser")
            .help("File owner not in passwd")
            .action(clap::ArgAction::SetTrue),
        Arg::new("nogroup")
            .long("nogroup")
            .help("File group not in group db")
            .action(clap::ArgAction::SetTrue),
        Arg::new("readable")
            .long("readable")
            .help("Readable by current user")
            .action(clap::ArgAction::SetTrue),
        Arg::new("writable")
            .long("writable")
            .help("Writable by current user")
            .action(clap::ArgAction::SetTrue),
        Arg::new("executable")
            .long("executable")
            .help("Executable by current user")
            .action(clap::ArgAction::SetTrue),
        Arg::new("links").long("links").value_name("N").help("Hard link count"),
        Arg::new("inum").long("inum").value_name("N").help("Inode number"),
        Arg::new("samefile")
            .long("samefile")
            .value_name("FILE")
            .help("Same inode and device as FILE"),
        Arg::new("mtime").long("mtime").value_name("N").help("Modification time in days (+N/-N/N)"),
        Arg::new("mmin").long("mmin").value_name("N").help("Modification time in minutes"),
        Arg::new("atime").long("atime").value_name("N").help("Access time in days"),
        Arg::new("amin").long("amin").value_name("N").help("Access time in minutes"),
        Arg::new("ctime").long("ctime").value_name("N").help("Change time in days"),
        Arg::new("cmin").long("cmin").value_name("N").help("Change time in minutes"),
        Arg::new("newer").long("newer").value_name("FILE").help("Modified more recently than FILE"),
        Arg::new("newerXY")
            .long("newerXY")
            .value_name("FILE")
            .help("Compare timestamps (X,Y: a/B/c/m/t)"),
        Arg::new("used")
            .long("used")
            .value_name("N")
            .help("Days since last access after status change"),
        Arg::new("lname").long("lname").value_name("PATTERN").help("Symlink target matches glob"),
        Arg::new("true").long("true").help("Always true").action(clap::ArgAction::SetTrue),
        Arg::new("false").long("false").help("Always false").action(clap::ArgAction::SetTrue),
    ]
    .into_iter()
    .map(|a| a.help_heading("Tests"))
    .collect()
}

fn action_args() -> Vec<Arg> {
    vec![
        Arg::new("print")
            .long("print")
            .help("Print path with newline (default)")
            .action(clap::ArgAction::SetTrue),
        Arg::new("print0")
            .long("print0")
            .help("Print path with NUL byte")
            .action(clap::ArgAction::SetTrue),
        Arg::new("printf")
            .long("printf")
            .value_name("FORMAT")
            .help("Formatted output (see FORMAT below)"),
        Arg::new("ls")
            .long("ls")
            .help("Output in ls -dils format")
            .action(clap::ArgAction::SetTrue),
        Arg::new("exec")
            .long("exec")
            .value_name("CMD ;")
            .help("Execute CMD for each file ({} = path)"),
        Arg::new("execdir")
            .long("execdir")
            .value_name("CMD ;")
            .help("Like -exec but runs from file's directory"),
        Arg::new("ok")
            .long("ok")
            .value_name("CMD ;")
            .help("Like -exec but prompts for confirmation"),
        Arg::new("delete")
            .long("delete")
            .help("Delete matched files (depth-first)")
            .action(clap::ArgAction::SetTrue),
        Arg::new("prune")
            .long("prune")
            .help("Don't descend into matched directories")
            .action(clap::ArgAction::SetTrue),
        Arg::new("quit")
            .long("quit")
            .help("Exit immediately on first match")
            .action(clap::ArgAction::SetTrue),
        Arg::new("fprint").long("fprint").value_name("FILE").help("Write paths to FILE"),
        Arg::new("fprint0").long("fprint0").value_name("FILE").help("Write paths to FILE with NUL"),
        Arg::new("fprintf")
            .long("fprintf")
            .value_name("FILE FORMAT")
            .help("Write formatted output to FILE"),
        Arg::new("fls").long("fls").value_name("FILE").help("Write ls output to FILE"),
    ]
    .into_iter()
    .map(|a| a.help_heading("Actions"))
    .collect()
}

fn operator_args() -> Vec<Arg> {
    vec![
        Arg::new("not")
            .long("not")
            .help("Negate the following expression (also: !)")
            .action(clap::ArgAction::SetTrue)
            .help_heading("Operators"),
    ]
}

// ANSI style helpers for after_help (clap styles only apply to arg definitions)
macro_rules! section {
    ($title:expr) => {
        concat!("\x1b[1;33m", $title, ":\x1b[0m")
    };
}

macro_rules! lit {
    ($text:expr) => {
        concat!("\x1b[1;32m", $text, "\x1b[0m")
    };
}

// Bash-like syntax highlighting for examples
macro_rules! cmd {
    ($text:expr) => {
        concat!("\x1b[1;37m", $text, "\x1b[0m")
    }; // bold white
}
macro_rules! flag {
    ($text:expr) => {
        concat!("\x1b[32m", $text, "\x1b[0m")
    }; // green
}
macro_rules! pat {
    ($text:expr) => {
        concat!("\x1b[33m", $text, "\x1b[0m")
    }; // yellow
}
macro_rules! ph {
    ($text:expr) => {
        concat!("\x1b[1;36m", $text, "\x1b[0m")
    }; // bold cyan
}
macro_rules! var {
    ($text:expr) => {
        concat!("\x1b[36m", $text, "\x1b[0m")
    }; // cyan
}
macro_rules! dim {
    ($text:expr) => {
        concat!("\x1b[2m", $text, "\x1b[0m")
    }; // dim
}

const AFTER_HELP: &str = concat!(
    // -- Operator syntax --
    section!("Operator syntax"),
    "\n",
    "  ",
    lit!("( EXPR )"),
    "          Group expressions\n",
    "  ",
    lit!("! EXPR"),
    " / ",
    lit!("-not"),
    "    Negate expression\n",
    "  ",
    lit!("EXPR -a EXPR"),
    "     AND (implicit between adjacent tests)\n",
    "  ",
    lit!("EXPR -o EXPR"),
    "     OR\n",
    "  ",
    lit!("EXPR , EXPR"),
    "      Evaluate both, return status of last\n",
    "\n",
    // -- Numeric arguments --
    section!("Numeric arguments"),
    "\n",
    "  ",
    lit!("+N"),
    "  greater than N    ",
    lit!("-N"),
    "  less than N    ",
    lit!("N"),
    "  exactly N\n",
    "\n",
    // -- printf format --
    section!("-printf format"),
    "\n",
    "  %p path  %f filename  %h parent dir  %d depth  %s size (bytes)\n",
    "  %m permissions (octal)  %M permissions (ls-style)  %u user  %g group\n",
    "  %l symlink target  %i inode  %n link count  %t mtime  %a atime  %c ctime\n",
    "  %T+  ISO 8601 mtime    \\n newline  \\t tab  \\0 NUL  %%  literal %\n",
    "\n",
    // -- exec / execdir --
    section!("-exec / -execdir"),
    "\n",
    "  -exec CMD {} \\;     Run CMD once per file ({} is replaced with path)\n",
    "  -exec CMD {} +      Run CMD with as many paths as possible (batched)\n",
    "  -execdir CMD {} \\;  Same but runs from the file's parent directory\n",
    "\n",
    // -- Examples --
    section!("Examples"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-name"),
    " ",
    pat!("'*.rs'"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-type"),
    " f ",
    flag!("-size"),
    " ",
    pat!("+10M"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-type"),
    " f ",
    flag!("-empty"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-maxdepth"),
    " 1 ",
    flag!("-type"),
    " d\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-mtime"),
    " -7 ",
    flag!("-name"),
    " ",
    pat!("'*.py'"),
    "\n",
    "\n",
    dim!("  # \\; = once per file, + = batch all paths into one call"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-name"),
    " ",
    pat!("'*.sh'"),
    " ",
    flag!("-exec"),
    " ",
    cmd!("chmod"),
    " +x ",
    ph!("{}"),
    " ",
    ph!("\\;"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-name"),
    " ",
    pat!("'*.txt'"),
    " ",
    flag!("-exec"),
    " ",
    cmd!("grep"),
    " -l TODO ",
    ph!("{}"),
    " ",
    ph!("+"),
    "\n",
    "  ",
    cmd!("find"),
    " . ",
    flag!("-name"),
    " ",
    pat!("'*.bak'"),
    " ",
    flag!("-execdir"),
    " ",
    cmd!("mv"),
    " ",
    ph!("{}"),
    " ",
    ph!("{}"),
    ".old ",
    ph!("\\;"),
    "\n",
    "\n",
    dim!("  # -print0 + xargs -0 for paths with spaces"),
    "\n",
    "  ",
    cmd!("find"),
    " /tmp ",
    flag!("-user"),
    " ",
    var!("$USER"),
    " ",
    flag!("-type"),
    " f ",
    flag!("-print0"),
    " | ",
    cmd!("xargs"),
    " -0 ",
    cmd!("rm"),
    "",
);

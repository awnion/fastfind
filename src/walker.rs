use std::io;
use std::io::Write;
use std::path::Path;

use regex::Regex;

use crate::cli::Config;
use crate::cli::FileType;
use crate::cli::glob_to_regex;

/// Walk the filesystem tree and print matching entries to stdout.
pub fn walk(config: &Config, stdout: &mut impl Write) -> io::Result<()> {
    let name_re = config
        .name_pattern
        .as_ref()
        .map(|p| {
            Regex::new(&glob_to_regex(p))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))
        })
        .transpose()?;

    for path in &config.paths {
        walk_jwalk(config, path, &name_re, stdout)?;
    }
    Ok(())
}

#[cfg(unix)]
fn write_path(stdout: &mut impl Write, path: &Path) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    stdout.write_all(path.as_os_str().as_bytes())?;
    stdout.write_all(b"\n")
}

#[cfg(not(unix))]
fn write_path(stdout: &mut impl Write, path: &Path) -> io::Result<()> {
    writeln!(stdout, "{}", path.display())
}

fn matches_entry(
    file_type_filter: Option<FileType>,
    name_re: &Option<Regex>,
    entry: &jwalk::DirEntry<((), ())>,
) -> bool {
    // type filter using d_type (no extra stat on Linux/macOS)
    if let Some(ft) = file_type_filter {
        let ok = match ft {
            FileType::File => entry.file_type.is_file(),
            FileType::Directory => entry.file_type.is_dir(),
            FileType::Symlink => entry.file_type.is_symlink(),
        };
        if !ok {
            return false;
        }
    }

    // name filter
    if let Some(re) = name_re {
        let name = entry.file_name.to_str().unwrap_or("");
        if !re.is_match(name) {
            return false;
        }
    }

    true
}

fn walk_jwalk(
    config: &Config,
    root: &Path,
    name_re: &Option<Regex>,
    stdout: &mut impl Write,
) -> io::Result<()> {
    let mut walker = jwalk::WalkDir::new(root).skip_hidden(false);

    if let Some(max) = config.max_depth {
        walker = walker.max_depth(max);
    }

    if config.min_depth > 0 {
        walker = walker.min_depth(config.min_depth);
    }

    let file_type = config.file_type;

    for entry in walker {
        let Ok(entry) = entry else { continue };

        if !matches_entry(file_type, name_re, &entry) {
            continue;
        }

        write_path(stdout, &entry.path())?;
    }

    Ok(())
}

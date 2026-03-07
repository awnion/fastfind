use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use regex::Regex;

use crate::cli::Config;
use crate::cli::FileType;
use crate::cli::glob_to_regex;

/// Walk the filesystem tree and print matching entries to stdout.
/// Returns true if at least one entry was printed.
pub fn walk(config: &Config, stdout: &mut impl Write) -> io::Result<bool> {
    let name_re = config
        .name_pattern
        .as_ref()
        .map(|p| {
            Regex::new(&glob_to_regex(p))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))
        })
        .transpose()?;

    let mut found = false;
    for path in &config.paths {
        found |= walk_dir(config, path, 0, &name_re, stdout)?;
    }
    Ok(found)
}

fn walk_dir(
    config: &Config,
    path: &Path,
    depth: usize,
    name_re: &Option<Regex>,
    stdout: &mut impl Write,
) -> io::Result<bool> {
    // respect maxdepth for traversal
    if config.max_depth.is_some_and(|max| depth > max) {
        return Ok(false);
    }

    let mut found = false;

    // check if current entry matches filters
    if depth >= config.min_depth && matches_filters(config, path, name_re) {
        writeln!(stdout, "{}", path.display())?;
        found = true;
    }

    // stop descending if maxdepth reached
    if config.max_depth.is_some_and(|max| depth >= max) {
        return Ok(found);
    }

    // descend into directories
    if path.is_dir() {
        let mut entries: Vec<_> =
            fs::read_dir(path)?.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        entries.sort();

        for entry in entries {
            found |= walk_dir(config, &entry, depth + 1, name_re, stdout)?;
        }
    }

    Ok(found)
}

fn matches_filters(config: &Config, path: &Path, name_re: &Option<Regex>) -> bool {
    // type filter
    if let Some(ft) = config.file_type {
        let ok = match ft {
            FileType::File => path.is_file(),
            FileType::Directory => path.is_dir(),
            FileType::Symlink => path.is_symlink(),
        };
        if !ok {
            return false;
        }
    }

    // name filter (matches against the file name component only)
    if let Some(re) = name_re {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return false,
        };
        if !re.is_match(name) {
            return false;
        }
    }

    true
}

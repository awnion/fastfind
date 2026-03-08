use std::fs;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::eval::EntryInfo;
use crate::eval::EntryType;
use crate::eval::EvalContext;
use crate::eval::QUIT_SIGNAL;
use crate::eval::flush_batch_exec;
use crate::eval::{self};
use crate::expr::Config;
use crate::expr::{self};

/// Walk the filesystem tree and evaluate expressions against entries.
pub fn walk(config: &Config, stdout: &mut impl Write) -> io::Result<()> {
    // Reset quit signal
    QUIT_SIGNAL.store(false, Ordering::Relaxed);

    let now = SystemTime::now();
    let daystart_time = if config.daystart { compute_daystart(now) } else { now };

    #[cfg(unix)]
    let root_devs: Vec<u64> = if config.xdev {
        config
            .paths
            .iter()
            .filter_map(|p| {
                use std::os::unix::fs::MetadataExt;
                fs::metadata(p).ok().map(|m| m.dev())
            })
            .collect()
    } else {
        Vec::new()
    };

    for (idx, path) in config.paths.iter().enumerate() {
        if QUIT_SIGNAL.load(Ordering::Relaxed) {
            break;
        }

        #[cfg(unix)]
        let root_dev = if config.xdev { root_devs.get(idx).copied() } else { None };
        #[cfg(not(unix))]
        let root_dev: Option<u64> = None;
        let _ = idx;

        let use_sequential = config.depth_first || expr::has_prune(&config.expr);
        if use_sequential {
            walk_sequential(config, path, path, &now, &daystart_time, root_dev, stdout)?;
        } else {
            walk_parallel(config, path, &now, &daystart_time, root_dev, stdout)?;
        }
    }

    Ok(())
}

/// Parallel walk using jwalk (default mode)
fn walk_parallel(
    config: &Config,
    root: &Path,
    now: &SystemTime,
    daystart_time: &SystemTime,
    #[allow(unused_variables)] root_dev: Option<u64>,
    stdout: &mut impl Write,
) -> io::Result<()> {
    let mut walker = jwalk::WalkDir::new(root)
        .skip_hidden(false)
        .follow_links(config.symlink_mode == crate::expr::SymlinkMode::Always);

    if let Some(max) = config.max_depth {
        walker = walker.max_depth(max);
    }

    if config.min_depth > 0 {
        walker = walker.min_depth(config.min_depth);
    }

    let mut ctx = EvalContext {
        now: *now,
        daystart: config.daystart,
        daystart_time: *daystart_time,
        starting_point: root,
        stdout,
        depth: 0,
        batch_exec: Vec::new(),
    };

    for entry in walker {
        if QUIT_SIGNAL.load(Ordering::Relaxed) {
            break;
        }

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if !config.ignore_readdir_race {
                    eprintln!("find: {e}");
                }
                continue;
            }
        };

        let path_buf = entry.path();

        // xdev check
        #[cfg(unix)]
        if let Some(dev) = root_dev {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = fs::symlink_metadata(&path_buf)
                && meta.dev() != dev
            {
                continue;
            }
        }

        let file_type = jwalk_file_type(&entry);
        let is_dir = matches!(file_type, EntryType::Directory);

        let mut info = EntryInfo {
            path: &path_buf,
            depth: entry.depth,
            file_type,
            metadata: None,
            is_dir,
            should_prune: false,
        };

        ctx.depth = entry.depth;

        let _ = eval::evaluate(&config.expr, &mut info, &mut ctx);
    }

    flush_batch_exec(&mut ctx);
    Ok(())
}

/// Sequential walk using std::fs (for -depth/-delete/-prune)
fn walk_sequential(
    config: &Config,
    root: &Path,
    starting_point: &Path,
    now: &SystemTime,
    daystart_time: &SystemTime,
    root_dev: Option<u64>,
    stdout: &mut impl Write,
) -> io::Result<()> {
    let mut ctx = EvalContext {
        now: *now,
        daystart: config.daystart,
        daystart_time: *daystart_time,
        starting_point,
        stdout,
        depth: 0,
        batch_exec: Vec::new(),
    };

    let result = walk_sequential_recursive(config, root, 0, root_dev, &mut ctx);
    flush_batch_exec(&mut ctx);
    result
}

fn walk_sequential_recursive(
    config: &Config,
    path: &Path,
    depth: usize,
    root_dev: Option<u64>,
    ctx: &mut EvalContext,
) -> io::Result<()> {
    if QUIT_SIGNAL.load(Ordering::Relaxed) {
        return Ok(());
    }

    if let Some(max) = config.max_depth
        && depth > max
    {
        return Ok(());
    }

    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            if !config.ignore_readdir_race {
                eprintln!("find: `{}`: {e}", path.display());
            }
            return Ok(());
        }
    };

    // xdev check
    #[cfg(unix)]
    if let Some(dev) = root_dev {
        use std::os::unix::fs::MetadataExt;
        if meta.dev() != dev {
            return Ok(());
        }
    }

    let is_dir = meta.is_dir();
    let file_type = entry_type_from_metadata(&meta);

    if config.depth_first {
        // Process children first, then this entry
        if is_dir && let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if QUIT_SIGNAL.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(entry) = entry {
                    walk_sequential_recursive(config, &entry.path(), depth + 1, root_dev, ctx)?;
                }
            }
        }

        if depth >= config.min_depth {
            let mut info = EntryInfo {
                path,
                depth,
                file_type,
                metadata: Some(meta),
                is_dir,
                should_prune: false,
            };
            ctx.depth = depth;
            let _ = eval::evaluate(&config.expr, &mut info, ctx);
        }
    } else {
        // Normal order: process this entry, then children (unless pruned)
        let mut should_descend = is_dir;

        if depth >= config.min_depth {
            let mut info = EntryInfo {
                path,
                depth,
                file_type,
                metadata: Some(meta),
                is_dir,
                should_prune: false,
            };
            ctx.depth = depth;
            let _ = eval::evaluate(&config.expr, &mut info, ctx);

            if info.should_prune {
                should_descend = false;
            }
        }

        if should_descend && let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if QUIT_SIGNAL.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(entry) = entry {
                    walk_sequential_recursive(config, &entry.path(), depth + 1, root_dev, ctx)?;
                }
            }
        }
    }

    Ok(())
}

fn jwalk_file_type(entry: &jwalk::DirEntry<((), ())>) -> EntryType {
    let ft = entry.file_type;
    if ft.is_file() {
        EntryType::File
    } else if ft.is_dir() {
        EntryType::Directory
    } else if ft.is_symlink() {
        EntryType::Symlink
    } else {
        #[cfg(unix)]
        {
            if let Ok(meta) = std::fs::symlink_metadata(entry.path()) {
                use std::os::unix::fs::MetadataExt;
                return EntryType::from_mode(meta.mode());
            }
        }
        EntryType::Unknown
    }
}

fn entry_type_from_metadata(meta: &fs::Metadata) -> EntryType {
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
            use std::os::unix::fs::MetadataExt;
            EntryType::from_mode(meta.mode())
        }
        #[cfg(not(unix))]
        EntryType::Unknown
    }
}

fn compute_daystart(now: SystemTime) -> SystemTime {
    let secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let day_secs = secs % 86400;
    UNIX_EPOCH + Duration::from_secs(secs - day_secs)
}

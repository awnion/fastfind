# GNU find coverage

Comparison of GNU `find` features vs `fastfind` current implementation.

Legend: `+` implemented, `-` not implemented, `~` partial

## Command-line options

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-H` | Don't follow symlinks except on command line | `+` |
| `-L` | Follow symbolic links | `+` |
| `-P` | Never follow symbolic links (default) | `+` |
| `-D debugopts` | Print diagnostic/debug information | `-` |
| `-Olevel` | Query optimization level (0-3) | `-` |

## Global options

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-maxdepth N` | Descend at most N levels | `+` |
| `-mindepth N` | Skip entries at depth less than N | `+` |
| `-depth` / `-d` | Process directory contents before the directory itself | `+` |
| `-mount` / `-xdev` | Don't descend into other filesystems | `+` |
| `-noleaf` | Don't optimize assuming directory link count | `+` |
| `-ignore_readdir_race` | Ignore errors if file vanishes between readdir and stat | `+` |
| `-noignore_readdir_race` | Opposite of above | `+` |
| `-files0-from file` | Read starting points from NUL-separated file | `-` |
| `-help` / `--help` | Show usage | `+` |
| `-version` / `--version` | Show version | `+` |

## Positional options

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-daystart` | Measure times from start of today | `+` |
| `-follow` | Follow symlinks (deprecated, use `-L`) | `-` |
| `-regextype type` | Choose regex dialect (emacs, posix-awk, etc.) | `+` |
| `-warn` / `-nowarn` | Enable/disable warnings | `+` |

## Tests - name/path matching

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-name pattern` | Match filename with glob | `+` |
| `-iname pattern` | Case-insensitive `-name` | `+` |
| `-path pattern` | Match full path with glob | `+` |
| `-ipath pattern` | Case-insensitive `-path` | `+` |
| `-wholename pattern` | Synonym for `-path` | `+` |
| `-iwholename pattern` | Case-insensitive `-wholename` | `+` |
| `-lname pattern` | Symlink target matches glob | `+` |
| `-ilname pattern` | Case-insensitive `-lname` | `+` |
| `-regex pattern` | Match full path with regex | `+` |
| `-iregex pattern` | Case-insensitive `-regex` | `+` |

## Tests - type

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-type c` | Match file type (f/d/l/b/c/p/s) | `+` |
| `-xtype c` | Like `-type` but checks symlink target | `+` |

## Tests - time

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-mtime n` | Modified n*24h ago | `+` |
| `-mmin n` | Modified n minutes ago | `+` |
| `-atime n` | Accessed n*24h ago | `+` |
| `-amin n` | Accessed n minutes ago | `+` |
| `-ctime n` | Status changed n*24h ago | `+` |
| `-cmin n` | Status changed n minutes ago | `+` |
| `-newer file` | Modified more recently than file | `+` |
| `-anewer file` | Accessed more recently than file's mtime | `+` |
| `-cnewer file` | Status changed more recently than file's mtime | `+` |
| `-newerXY ref` | Timestamp X newer than ref's timestamp Y | `~` (no literal timestamp via `t`) |
| `-used n` | Accessed n days after status change | `+` |

## Tests - size/content

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-size n[cwbkMG]` | File uses n units of space | `+` |
| `-empty` | File/directory is empty | `+` |

## Tests - permissions

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-perm mode` | Exact permission match | `~` (octal only, no symbolic) |
| `-perm -mode` | All bits set | `~` (octal only, no symbolic) |
| `-perm /mode` | Any bits set | `~` (octal only, no symbolic) |
| `-readable` | Readable by current user | `+` |
| `-writable` | Writable by current user | `+` |
| `-executable` | Executable/searchable by current user | `+` |

## Tests - ownership

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-user name` | Owned by user | `+` |
| `-uid n` | Numeric user ID | `+` |
| `-group name` | Belongs to group | `+` |
| `-gid n` | Numeric group ID | `+` |
| `-nouser` | No user for file's UID | `+` |
| `-nogroup` | No group for file's GID | `+` |

## Tests - filesystem/inode

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-fstype type` | File on filesystem of type | `+` |
| `-inum n` | File has inode number n | `+` |
| `-samefile name` | Same inode as name | `+` |
| `-links n` | File has n hard links | `+` |

## Tests - misc

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-true` | Always true | `+` |
| `-false` | Always false | `+` |
| `-context pattern` | SELinux context matches glob | `-` |

## Actions

| Flag | GNU find description | fastfind |
|------|---------------------|----------|
| `-print` | Print path + newline (default) | `+` |
| `-print0` | Print path + NUL | `+` |
| `-printf format` | Formatted output | `~` (no width/flag modifiers) |
| `-fprint file` | Print path to file | `+` |
| `-fprint0 file` | Print path + NUL to file | `+` |
| `-fprintf file fmt` | Formatted output to file | `~` (no width/flag modifiers) |
| `-ls` | List in `ls -dils` format | `+` |
| `-fls file` | `-ls` output to file | `+` |
| `-exec cmd ;` | Run command per file | `+` |
| `-exec cmd {} +` | Run command with batched args | `+` |
| `-execdir cmd ;` | Like `-exec` but from file's dir | `+` |
| `-execdir cmd {} +` | Batched `-execdir` | `+` |
| `-ok cmd ;` | `-exec` with confirmation prompt | `+` |
| `-okdir cmd ;` | `-execdir` with confirmation prompt | `+` |
| `-delete` | Delete matched files | `+` |
| `-prune` | Don't descend into directory | `+` |
| `-quit` | Exit immediately | `+` |

## Operators

| Operator | GNU find description | fastfind |
|----------|---------------------|----------|
| `( expr )` | Grouping | `+` |
| `! expr` / `-not` | Negation | `+` |
| `expr1 expr2` / `-a` / `-and` | Logical AND (implicit) | `+` |
| `-o` / `-or` | Logical OR | `+` |
| `,` | List (both evaluated) | `+` |

## Numeric argument prefixes

| Prefix | GNU find description | fastfind |
|--------|---------------------|----------|
| `+n` | Greater than n | `+` |
| `-n` | Less than n | `+` |
| `n` | Exactly n | `+` |

## Summary

| Category | Total | Implemented | Coverage |
|----------|-------|-------------|----------|
| Command-line options | 5 | 3 | 60% |
| Global options | 10 | 9 | 90% |
| Positional options | 4 | 3 | 75% |
| Tests - name/path | 10 | 10 | 100% |
| Tests - type | 2 | 2 | 100% |
| Tests - time | 11 | 10 + 1 partial | 95% |
| Tests - size/content | 2 | 2 | 100% |
| Tests - permissions | 6 | 3 + 3 partial | 75% |
| Tests - ownership | 6 | 6 | 100% |
| Tests - fs/inode | 4 | 4 | 100% |
| Tests - misc | 3 | 2 | 67% |
| Actions | 17 | 15 + 2 partial | 94% |
| Operators | 5 | 5 | 100% |
| Numeric prefixes | 3 | 3 | 100% |
| **Total** | **88** | **80** | **~91%** |

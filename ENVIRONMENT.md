# Environment variables and CLI flags

## CLI flags

### Filters

| Flag | Description |
| --- | --- |
| `-type f\|d\|l` | Filter by entry type: file, directory, or symlink |
| `-name PATTERN` | Match filename against a shell glob pattern |
| `-maxdepth N` | Descend at most N directory levels below the starting point |
| `-mindepth N` | Do not apply tests or actions at levels less than N |

### Actions

| Flag | Description |
| --- | --- |
| `-print` | Print the full path of each matching entry (default action) |

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Error (bad arguments, path not found, etc.) |

# GNU find compatibility

Flags and options not yet supported by fastfind.

**Perf impact** — potential performance degradation: 🟢 none, 🟡 minor, 🔴 significant.
**Complexity** — implementation effort: 🟢 easy, 🟡 moderate, 🔴 hard.

## Leading / debug options

| Flag | Description | Perf impact | Complexity |
|------|-------------|:-----------:|:----------:|
| `-D debugopts` | Print diagnostic information | 🟢 | 🟡 |
| `-Olevel` | Query optimization level (0-3) | 🟢 | 🔴 |

## Global options

| Flag | Description | Perf impact | Complexity |
|------|-------------|:-----------:|:----------:|
| `-files0-from FILE` | Read starting points from NUL-delimited file | 🟢 | 🟢 |
| `-follow` | Deprecated synonym for `-L` | 🟢 | 🟢 |

## Tests

| Flag | Description | Perf impact | Complexity |
|------|-------------|:-----------:|:----------:|
| `-perm MODE` (symbolic) | Symbolic permission modes (e.g. `u+x`, `g=rw`) | 🟢 | 🟡 |
| `-context PATTERN` | SELinux security context match | 🟢 | 🟡 |
| `-newerXY` with `t` ref | Compare against literal timestamp string | 🟢 | 🟡 |

## Actions

| Flag | Description | Perf impact | Complexity |
|------|-------------|:-----------:|:----------:|
| `-printf` width/flags | Field width and formatting flags (e.g. `%20f`, `%-10p`) | 🟢 | 🟡 |

## Environment

| Variable | Description | Perf impact | Complexity |
|----------|-------------|:-----------:|:----------:|
| `POSIXLY_CORRECT` | Strict POSIX compliance mode | 🟢 | 🟡 |
| `TZ` | Timezone for `-printf` time directives | 🟢 | 🟢 |

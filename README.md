```
nix run github:marcuswhybrow/flake-updates -- --flake /path/to/flake-dir
```

```
❯ flake-updates --help
Usage: flake-updates [OPTIONS]

Options:
  -f, --flake <FLAKE>    Path to a nix flake.lock file, or it's parent directory [default: .]
  -t, --ttl <TTL>        The "Time To Live" of cached GitHub API requests before being considered stale. Caching GitHub API requests avoids rate limiting denials at the cost of data freshness [default: 60] [aliases: poll] [short aliases: p]
  -o, --output <OUTPUT>  Output string format ("%s is replaced with the number of updates")
  -d, --defer            Immediately return any currently cached data regardless of --ttl, and asynchronously consider regenerating the cache as a background task
  -h, --help             Print help
  -V, --version          Print version
```

Works great for a custom [Waybar](https://github.com/Alexays/Waybar) module:

```
{
    "custom/updates": {
        "exec": "flake-updates --flake /path/to/flake-dir --output '%s updates'",
        "interval": 1
    }
}
```

- Makes 1 GitHub API call per input by default.
- `flake-updates --help` for more options.
- Top tip, `flake-updates --ttl 0` busts the cache and always calls GitHub.


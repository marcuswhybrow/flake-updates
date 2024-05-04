```
nix run github:marcuswhybrow/flake-updates -- --flake /path/to/flake-dir
```

```
❯ flake-updates --help
Usage: flake-updates [OPTIONS]

Options:
  -f, --flake <FLAKE>    Path to a nix flake.lock file, or it's parent directory [default: .]
  -p, --poll <POLL>      How often to check GitHub for updates [default: 60]
  -o, --output <OUTPUT>  Output string format ("%s is replaced with the number of updates")
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
- Top tip, `flake-updates --poll 0` busts the cache and always calls GitHub.


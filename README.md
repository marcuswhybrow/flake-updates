```
nix run github:marcuswhybrow/flake-updates -- --flake /path/to/flake-dir
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
- Top tip, `flake-updates --poll 0` busts the cache to always calls GitHub.


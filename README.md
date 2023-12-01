```
nix run github:marcuswhybrow/flake-updates -- /path/to/flake-dir input-name
```

Works great for a custom [Waybar](https://github.com/Alexays/Waybar) module:

```
{
    "custom/updates": {
        "exec": "flake-updates /path/to/flake-dir input-name",
        "interval": 1
    }
}
```

- Makes a maximum of 2 calls per hour to the GitHub API.

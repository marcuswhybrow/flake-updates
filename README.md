Works great for a custom [Waybar](https://github.com/Alexays/Waybar) module:

```
{
    "custom/updates": {
        "exec": "nixpkgs-diff /path/to/nix/flake",
        "interval": 1
    }
}
```

- Makes a maximum of 2 calls per hour to the GitHub API.

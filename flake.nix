{
  description = "Rust project packaged using Naersk and rust-overlay";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = inputs: let 
    pkgs = import inputs.nixpkgs { 
      system = "x86_64-linux"; 
      overlays = [ 
        inputs.cargo2nix.overlays.default
      ];
    };

    rustPkgs = pkgs.rustBuilder.makePackageSet {
      rustVersion = "1.74.0";
      packageFun = import ./Cargo.nix;
    };
  in {
    packages.x86_64-linux.flake-updates = rustPkgs.workspace.flake-updates {};
    packages.x86_64-linux.default = inputs.self.packages.x86_64-linux.flake-updates;

    devShells.x86_64-linux.default = rustPkgs.workspaceShell {
      name = "flake-updates-shell";
      packages = [ 
        inputs.cargo2nix.packages.x86_64-linux.cargo2nix
        pkgs.bacon
      ];
    };
  };
}

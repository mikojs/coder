# Coder

A CLI tool for syncing git repositories between local and remote servers using git bundles.

This tool is useful when you need to sync code with a remote server that doesn't have direct git remote access (e.g., air-gapped environments, restricted networks).

## Installation

### Cargo

```bash
cargo install --path .
```

### Nix

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    miko-coder.url = "github:mikojs/coder";
  };

  outputs = { nixpkgs, miko-coder, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      ...
      modules = [
        ({ pkgs, ... }: {
          nixpkgs.overlays = [ miko-coder.overlays.default ];
          environment.systemPackages = [ pkgs.miko-coder ];
        })
      ];
    };
  };
}
```

## Usage

### Sync

Sync local repository with a git bundle file:

```bash
coder sync <BUNDLE>
```

This command will:
- Parse branches from the bundle file
- Remove local branches that don't exist in the bundle
- Add new branches from the bundle
- Update all branches with changes from the bundle

### Push

Push local repository to a remote server:

```bash
coder push <SSH_URL> <DIRECTORY>
```

This command will:
1. Create a git bundle from all local branches
2. Transfer the bundle to the remote server via SCP
3. Run `coder sync` on the remote server
4. Clean up temporary files

### Pull

Pull repository from a remote server:

```bash
coder pull <SSH_URL> <DIRECTORY>
```

This command will:
1. Create a git bundle on the remote server
2. Transfer the bundle to local via SCP
3. Run sync locally to update branches
4. Clean up temporary files

## Shell Completion

Generate shell completion scripts:

```bash
coder --generate <bash|zsh|fish> > shell init script
```

## Requirements

- Git
- SSH/SCP (for push/pull commands)
- `coder` must be installed on the remote server (for push command)

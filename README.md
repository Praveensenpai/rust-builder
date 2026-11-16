# rbuild

Build your Rust app on a fast cloud computer instead of your own.

`rbuild` sends your project to GitHub Codespaces, builds it there, and brings the finished app back to you. This helps keep your computer cool and fast.

## Install (easy)

Open Terminal and run (works from any folder):

```bash
curl -fsSL https://raw.githubusercontent.com/Praveensenpai/rust-builder/main/install.sh | bash
```

After install, restart your terminal if needed, then check:

```bash
rbuild --help
```

To update later, run the same install command again.

## Before you start

You need:
- A GitHub account
- [GitHub CLI](https://cli.github.com/) (`gh`) installed and logged in: `gh auth login`
- GitHub Codespaces available on your account

## How to use

1. Open your Rust project folder in Terminal (the folder with `Cargo.toml`).
2. Run:

```bash
rbuild
```

3. When it finishes, your built app is saved in:
   - `target/remote/release/` (inside your project)

### Handy commands

```bash
rbuild            # build your app
rbuild -i         # build and put it in ~/.local/bin
rbuild --check    # quick check (no app file made)
rbuild --clippy   # extra code checks
rbuild --test     # run tests
rbuild status     # see builder status
rbuild stop       # stop the cloud builder to save free time
```

## Problems?

- **"command not found: rbuild"**: add this to `~/.bashrc` or `~/.zshrc` and reopen Terminal:
  ```bash
  export PATH="$HOME/.local/bin:$PATH"
  ```
- **GitHub login issues**: run `gh auth login` again.
- **Need more details**: see [SKILL.md](SKILL.md) or run `rbuild --help`.

## License

MIT

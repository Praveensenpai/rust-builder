# ⚡ 遠隔構築 · rbuild (rust-builder)

> **Compile Rust projects on remote 4-vCPU GitHub Codespaces with zero local CPU load and persistent SSD caching.**

[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS-1793D1?style=flat-square&logo=linux&logoColor=white)](https://github.com/Praveensenpai/rust-builder)
[![Compiler](https://img.shields.io/badge/Compiler-Rust%202021-DEA584?style=flat-square&logo=rust&logoColor=white)](https://rust-lang.org)
[![Linker](https://img.shields.io/badge/Linker-mold-a6e3a1?style=flat-square)](https://github.com/rui314/mold)
[![Quota](https://img.shields.io/badge/Quota-60h%20Free%20Tier-cba6f7?style=flat-square)](https://github.com/features/codespaces)

Compiling heavy Rust crates pegs your CPU at 100%, spins your laptop fans, and drains your battery. `rbuild` streams your source code over an encrypted SSH pipe into a dedicated GitHub Codespace (4 vCPUs, 16GB RAM) backed by persistent SSD storage, the **mold** linker, and **sccache**, returning the compiled binary directly to your machine.

---

## 🔄 Architecture & Flow

```text
┌───────────────────────────────────────────────┐
│  💻 Local Machine                             │
│  $ rbuild                                     │
└───────────────────────┬───────────────────────┘
                        │ 1. Stream source tarball (<1s)
                        │    (skips local target/ & .git/)
                        ▼
┌───────────────────────────────────────────────┐
│  ☁️ Remote Codespace (4 vCPUs · 16GB RAM)     │
│  • mold modern linker (<1s link times)        │
│  • sccache compiler cache                     │
│  • Persistent SSD (warm target/ directories)  │
│  • Auto-purges oldest targets if disk > 80%   │
└───────────────────────┬───────────────────────┘
                        │ 2. Stream binary back over SSH
                        ▼
┌───────────────────────────────────────────────┐
│  📦 target/remote/release/<binary>            │
│  (or ~/.local/bin/ with -i)                   │
└───────────────────────────────────────────────┘
```

---

## 🪄 One-Liner Install

```bash
cargo install --path .
# or install directly to ~/.local/bin
install -Dm755 target/release/rbuild ~/.local/bin/rbuild
```

---

## 🚀 Quick Usage

```bash
# Build current local Rust project in release mode
rbuild

# Build and copy directly to ~/.local/bin/
rbuild -i

# Build specific binary target in workspace
rbuild --bin my-tool

# Check status of builder Codespaces and storage
rbuild status

# Stop Codespace to conserve free tier hours
rbuild stop
```

---

## ⚡ Cache & Storage Management

| Feature | Mechanism | Benefit |
|---|---|---|
| **SSD Persistence** | Remote `/workspaces` volume | Dependencies are downloaded and compiled only once |
| **mold Linker** | System clang + `-fuse-ld=mold` | Linking completes in sub-seconds rather than 15–30s |
| **sccache** | Object file cache | Repeated builds with identical code hit 100% cache |
| **Auto-Purge** | Proactive capacity daemon | When disk exceeds 80%, oldest `target/` dirs are pruned |
| **Auto-Stop** | 10-minute idle timeout | Codespace shuts down when idle, saving monthly quota |

---

## 📜 License

MIT © Praveen Senpai ([@Praveensenpai](https://github.com/Praveensenpai))

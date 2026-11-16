# CODEBASE.md: rust-builder Semantic Digest

> **Notice**: AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow
```text
Entrypoint ──> CLI/Parser ──> Domain Logic ──> Infra/IO
```

## 2. Global Constraints & Architecture Patterns
- **Primary Language**: Rust 2021 edition
- **Architectural Paradigm**: Role-based (domain/, infra/, api/cli/, tui/)
- **Hard Constraints**: <400 lines/file, <60 lines/fn, zero production unwrap(), 0 warnings.
- **Target Distribution**: Linux x86_64 standalone binary

## 3. Module & Interface Skeleton

### `src/cli.rs` (Role: cli, Lines: 111)
- **Responsibility**: Core cli logic in src/cli.rs
- **Imports**: use clap :: { Parser , Subcommand } , use std :: path :: PathBuf 
- **Types & Enums**:
  ```rust
  pub struct Cli
  pub enum Commands
  ```

### `src/domain/config.rs` (Role: domain, Lines: 56)
- **Responsibility**: Core domain logic in src/domain/config.rs
- **Types & Enums**:
  ```rust
  pub struct BuildConfig
  pub enum VerifyKind { Check, Clippy, Test }
  ```
- **Public Functions & Signatures**:
  ```rust
  fn cargo_subcommand (self) -> & 'static str
  ```

### `src/domain/project.rs` (Role: domain, Lines: 160)
- **Responsibility**: Core domain logic in src/domain/project.rs
- **Imports**: use anyhow :: { Context , Result , bail } , use serde :: Deserialize , use sha2 :: { Digest , Sha256 } , use std :: fs , use std :: path :: { Path , PathBuf } , use walkdir :: WalkDir 
- **Types & Enums**:
  ```rust
  pub struct ProjectManifest
  ```
- **Public Functions & Signatures**:
  ```rust
  fn discover_project (start_path : & Path) -> Result < ProjectManifest >
  fn compute_source_hash (root_dir : & Path) -> Result < String >
  ```

### `src/domain.rs` (Role: domain, Lines: 2)
- **Responsibility**: Core domain logic in src/domain.rs

### `src/infra/codespace.rs` (Role: infra, Lines: 159)
- **Responsibility**: Core infra logic in src/infra/codespace.rs
- **Imports**: use anyhow :: { Context , Result , bail } , use indicatif :: ProgressBar , use serde :: Deserialize , use std :: process :: Command , use std :: time :: Duration , use crate :: domain :: config :: DEFAULT_IDLE_TIMEOUT 
- **Types & Enums**:
  ```rust
  pub struct CodespaceItem
  ```
- **Public Functions & Signatures**:
  ```rust
  fn list_codespaces () -> Result < Vec < CodespaceItem > >
  fn find_builder_codespace (target : & str) -> Result < Option < CodespaceItem > >
  fn ensure_builder_codespace (target_name : & str , repo : & str , spinner : & ProgressBar ,) -> Result < CodespaceItem >
  fn exec_ssh (name : & str , command : & str) -> Result < String >
  fn stop_codespace (name : & str) -> Result < () >
  ```

### `src/infra/download.rs` (Role: infra, Lines: 60)
- **Responsibility**: Core infra logic in src/infra/download.rs
- **Imports**: use anyhow :: { Context , Result , bail } , use indicatif :: ProgressBar , use std :: fs , use std :: os :: unix :: fs :: PermissionsExt , use std :: path :: { Path , PathBuf } , use std :: process :: { Command , Stdio } 
- **Public Functions & Signatures**:
  ```rust
  fn retrieve_binary (codespace_name : & str , remote_path : & str , bin_name : & str , dest_dir : & Path , install_to_bin : bool , spinner : & ProgressBar ,) -> Result < (PathBuf , u64) >
  ```

### `src/infra/remote_builder.rs` (Role: infra, Lines: 197)
- **Responsibility**: Core infra logic in src/infra/remote_builder.rs
- **Imports**: use anyhow :: { Context , Result , bail } , use indicatif :: ProgressBar , use std :: process :: { Command , Stdio } , use crate :: domain :: config :: { BuildConfig , VerifyKind } 
- **Public Functions & Signatures**:
  ```rust
  fn execute_remote_build (remote_dir : & str , bin_name : & str , config : & BuildConfig , spinner : & ProgressBar ,) -> Result < String >
  fn execute_remote_verify (remote_dir : & str , kind : & VerifyKind , config : & BuildConfig , spinner : & ProgressBar ,) -> Result < () >
  ```

### `src/infra/storage.rs` (Role: infra, Lines: 38)
- **Responsibility**: Core infra logic in src/infra/storage.rs
- **Imports**: use anyhow :: { Context , Result } , use indicatif :: ProgressBar , use std :: process :: Command , use crate :: domain :: config :: STORAGE_CLEANUP_THRESHOLD_PERCENT 
- **Public Functions & Signatures**:
  ```rust
  fn check_and_clean_storage (codespace_name : & str , spinner : & ProgressBar) -> Result < () >
  ```

### `src/infra/sync.rs` (Role: infra, Lines: 58)
- **Responsibility**: Core infra logic in src/infra/sync.rs
- **Imports**: use anyhow :: { Context , Result , bail } , use indicatif :: ProgressBar , use std :: path :: Path , use std :: process :: { Command , Stdio } , use crate :: domain :: config :: REMOTE_PROJECTS_DIR 
- **Public Functions & Signatures**:
  ```rust
  fn sync_project_source (codespace_name : & str , project_name : & str , local_dir : & Path , spinner : & ProgressBar ,) -> Result < String >
  ```

### `src/infra.rs` (Role: infra, Lines: 11)
- **Responsibility**: Core infra logic in src/infra.rs
- **Imports**: pub use codespace :: * , pub use download :: * , pub use remote_builder :: * , pub use storage :: * , pub use sync :: * 

### `src/main.rs` (Role: general, Lines: 269)
- **Responsibility**: Core general logic in src/main.rs
- **Imports**: use anyhow :: { Context , Result } , use clap :: Parser , use console :: style , use std :: process :: ExitCode , use std :: time :: Instant , use cli :: { Cli , Commands } , use domain :: config :: { BuildConfig , DEFAULT_CODESPACE_NAME , VerifyKind } , use domain :: project :: { ProjectManifest , discover_project } 

### `src/ui.rs` (Role: tui, Lines: 59)
- **Responsibility**: Core tui logic in src/ui.rs
- **Imports**: use console :: style , use indicatif :: { ProgressBar , ProgressStyle } , use std :: time :: Duration 
- **Public Functions & Signatures**:
  ```rust
  fn create_spinner (msg : & str) -> ProgressBar
  fn print_step (title : & str , detail : & str)
  fn print_success (msg : & str)
  fn print_error (msg : & str)
  fn print_info (title : & str , content : & str)
  fn format_duration (dur : Duration) -> String
  fn format_bytes (bytes : u64) -> String
  ```

## 4. Execution Lifecycle Trace
1. **Startup**: Entrypoint parses CLI flags & dispatches command.
2. **Execution**: Core domain logic processes inputs and evaluates rules.
3. **Persistence / I/O**: Domain logic calls infra for disk/terminal I/O.
4. **Exit**: Graceful termination with standard exit codes.

## 5. Verification Commands
```bash
# Build
cargo build --release --target x86_64-unknown-linux-gnu

# Test
cargo test --all-targets

# Lint & Format
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## 6. Recent Iteration Changes
- **2026-11-16**: Ran full verification suite (check/clippy/fmt/test/release). Fixed formatting drift via `cargo fmt` across `src/cli.rs`, `src/domain/*.rs`, `src/infra/*.rs`, `src/main.rs`. No logic changes.
- **2026-11-17**: Added remote verification mode: `--check`, `--clippy`, `--test` flags run `cargo check/clippy/test --all-targets` on the Codespace runner via new `infra::execute_remote_verify` + `domain::VerifyKind` (clippy adds `-- -D warnings`). Added shell-quote tokenization for remote cargo args (fixes latent space-in-flag-value bug) and 4 unit tests in `src/infra/remote_builder.rs`.

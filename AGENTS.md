# Testing your changes

- Build the Rust crates with `cargo build` (from this directory or the parent repo root).
- Run the Rust tests with `cargo test --workspace`.
- Lint Rust code with `cargo clippy --workspace`.
  Don't disable lints just to make the linter happy. ALWAYS try to fix the code first.
- Run `cargo fmt --check` to verify formatting (fix with `cargo fmt`).

When the test suite fails and you want to test potential fixes, try running only the affected
tests first, one by one:

```bash
cargo test -p ah-auth test_name
cargo test -p ah-billing test_name
cargo test -p ah-platform-api test_name
```

## Managing dependencies

This repo is a git submodule of the [agent-harbor](https://github.com/blocksense-network/agent-harbor) monorepo. The parent repo provides the nix flake, dev shell (via direnv), and `just` command runner.

- **Rust crates** are managed with Cargo. The workspace is defined in `Cargo.toml` at this repo's root.
- **TypeScript/Node** dependencies are managed with the parent repo's yarn (PnP mode).
- **Nix dev shell**: enter via `direnv` in the parent repo root. Rust itself is pinned in the parent's nix flake.
- After adding new dependencies, always make sure that you are adding a recent version.
- When you are facing issues with a dependency, make sure to study its implementation files that you can find in the local nix store, the local cargo cache, etc.

## Code quality guidelines

- ALWAYS strive to achieve high code quality.
- ALWAYS write secure code.
- ALWAYS make sure the code is well tested and edge cases are covered. Design the code for testability and be extremely thorough.
- ALWAYS write defensive code and make sure all potential errors are handled.
- ALWAYS strive to write highly reusable code with routines that have high fan in and low fan out.
- ALWAYS keep the code DRY.
- ALWAYS research the problem domain and the tech stack being used to ensure you are following the best practices.
- Aim for low coupling and high cohesion. Encapsulate and hide implementation details.
- When creating executable, ALWAYS make sure the functionality can also be used as a library.
  To achieve this, avoid global variables, raise/return errors instead of terminating the program, and think whether the use case of the library requires more control over logging
  and metrics from the application that integrates the library.

## Code commenting guidelines

- Document public APIs and complex modules using standard code documentation conventions.
- Comment the intention behind you code extensively. Omit comments only for very obvious
  facts that almost any developer would know.
- Maintain the comments together with the code to keep them meaningful and current.
- When the code is based on specific formats, standards or well-specified behavior of
  other software, always make sure to include relevant links (URLs) that provide the
  necessary technical details.

### Test writing guidelines

- Each test MUST create a unique log file capturing its full output.
- On success: tests print minimal output to keep logs out of AI context windows.
- On failure: the test runner prints the path and file size of the relevant log(s) so developers (or agents) can open them directly without flooding the console or context.
- Rationale: preserves context-budget for AI tools by avoiding large inline logs, while retaining full fidelity in files.
- Every bug/regression/issue fix MUST ship with at least one automated regression test in the same change.
- NEVER cheat with the tested assertions in order to satisfy a specified Verification requirement (e.g. one from a .status.md file)
- ALWAYS prefer creating automated tests instead of managing multiple client and server processes interactively.

## Writing git commit messages

- You MUST use multiline git commit messages using heredoc syntax.
- Use the conventional commits style for the first line of the commit message.
- Use the summary section of your final response as the remaining lines in the commit message.

# Agent Harbor Platform

Commercial platform layer for [Agent Harbor](https://github.com/blocksense-network/agent-harbor) -- authentication, billing, team management, and the platform API.

## Structure

### Rust crates (`crates/`)

| Crate | Description |
|---|---|
| `ah-auth` | Authentication primitives -- JWT, OAuth, password hashing, rate limiting, token management |
| `ah-billing` | Stripe integration -- checkout, webhooks, usage metering, dunning |
| `ah-platform-api` | HTTP API -- auth, executor, team, org, and invitation handlers with middleware |

### TypeScript (`src/`)

| Directory | Description |
|---|---|
| `viewmodel/` | View-models for admin, auth, billing, onboarding, security, teams, and more |
| `stories/` | Storybook stories for admin, auth, onboarding, and portal UIs |
| `nav/` | Platform navigation items |

## Development

This repo is a **git submodule** of the [agent-harbor](https://github.com/blocksense-network/agent-harbor) monorepo. Development is done from the parent repo root, which provides the nix dev shell, `just` targets, and pre-commit hooks.

```bash
# From the parent repo root
cd platform/

# Build
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace
cargo fmt --check
```

## License

Proprietary -- all rights reserved by Schelling Point Labs Inc.

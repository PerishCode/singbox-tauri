# Repository Guidelines

## Scope
- Thin Tauri client for sing-box.
- MVP focuses on TUN integration, encrypted HTTP subscription fetch, and logs.

## Layout
- `src/`: Tauri frontend.
- `src-tauri/`: Rust backend, process control, local state.

## Local Commands
- `./scripts/prepare.sh`
- `pnpm install`
- `pnpm tauri dev`
- `pnpm build`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `./scripts/dev.sh {start|stop|restart|attach|logs|status}`

## Style
- Keep UI thin; push logic into Rust commands/services.
- Prefer small modules and explicit state boundaries.
- Keep secrets local; never commit private keys or sample decrypted payloads.

## Iteration
- Start with MVP only: start/stop, fetch/decrypt subscription, tail logs.
- Avoid generic proxy-client features unless they directly support the MVP.
- Use manual restarts for local dev; do not rely on watch mode.
- Keep runtime preparation unified through `SINGBOX_TAURI_RUNTIME_ROOT_PATH`.

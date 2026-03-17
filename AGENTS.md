# Repository Guidelines

## Scope
- Thin Tauri client for sing-box.
- MVP focuses on TUN integration, encrypted HTTP subscription fetch, and logs.

## Layout
- `src/`: Tauri frontend.
- `src-tauri/`: Rust backend, process control, local state.

## Runtime Subscription Layout
- `subscriptions/subscription.json.age`: last fetched encrypted artifact.
- `subscriptions/subscription.json`: decrypted artifact for local inspection.
- `subscriptions/active-config.json`: staged sing-box config produced by the subscription pipeline.
- `config/runtime.json`: runtime config currently handed to sing-box after apply/restart.

## Local Commands
- `./scripts/prepare.sh`
- `pnpm install`
- `pnpm openapi:gen`
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
- Run the local app first before `pnpm openapi:gen`; the OpenAPI schema is fetched from the local control server.

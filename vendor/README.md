# Winit 0.31 compatibility patches

The workspace uses `richerfu/winit` at `c9ee2bff8a79aacb8068546e7150197693d35a5a`
(0.31.0-beta.3), including its separate OpenHarmony backend. The full winit source is
not vendored. `winit-core` and `dpi` are patched to the same revision so the facade,
backend and adapters share the same Rust types.

| Directory | Source | Local changes |
| --- | --- | --- |
| `winit-ohos` | [ohos-rs/winit-ohos `feat/egui-ohos-adaptation`](https://github.com/ohos-rs/winit-ohos/tree/d57ae21d1a23a5b194fbe58af6f61f65579f97d8) | Sources are unchanged. The manifest uses registry requirements resolved through the root patches; the README links to this workspace. |
| `openharmony-ability` | [richerfu/openharmony-ability](https://github.com/richerfu/openharmony-ability/tree/06e041fa81862410bee03b7856dc09a489f49639) | Typed RGBA custom cursors and runtime 2in1 icons, native drag previews/results and Copy/Move negotiation, pen metadata; retained MIT/Apache licences. |
| `ohos-arkui-binding` | crates.io 0.4.1 | Owned native drag completion result, operation and preview hotspot. |
| `ohos-xcomponent-binding` | crates.io 0.5.1 | Query tool type by touch-point index, capture tilt. |
| `glutin-winit` | [rust-windowing/glutin](https://github.com/rust-windowing/glutin/tree/940a394cc205f152eb12076feff59f1eaaff4f7a/glutin-winit) | Standalone manifest, `CreateWindowError`, exclude OHOS from Unix desktop backends. |
| `accesskit_winit` | [AccessKit 0.29.2](https://crates.io/crates/accesskit_winit/0.29.2) | Window/event-loop traits, callback event delivery, surface geometry, exclude OHOS from AT-SPI. Native OHOS provider registration and per-window lifecycle. Keeps the existing AccessKit 0.21 schema. |
| `accesskit_ohos` | [richerfu/accesskit](https://github.com/richerfu/accesskit/tree/ea2d8c4) | Backport the OHOS provider adapter to AccessKit 0.21 / consumer 0.31; preserve licenses. |

The ability revision is `06e041fa81862410bee03b7856dc09a489f49639`, aligned with
[`gpui-ohos`'s `feat/pr82-ability-adaptation` branch](https://github.com/ohos-rs/gpui-ohos/tree/feat/pr82-ability-adaptation).
Rust and ArkTS use the same locally patched Ability source. ArkUI 0.4.1 and
XComponent 0.5.1 are vendored from crates.io for the owned drag-completion
callback and pen tool-index/tilt fixes; IME 0.3.1 and accessibility 0.1.2 remain
registry dependencies. The plugins and renderers share the same native types.

These are compatibility patches for unreleased APIs. Update each directory from its
upstream when equivalent fixes land, retaining its license. Applications consuming
this fork from another Cargo workspace must also copy the root `[patch.crates-io]`
entries (and these vendor paths); Cargo does not inherit dependency workspaces' patches.

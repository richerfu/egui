# winit-ohos

Winit's OpenHarmony backend, driven by `openharmony-ability` lifecycle callbacks.

## Dependency policy

This crate tracks the matching released `winit-core` beta from crates.io. The current Ability/N-API
toolchain requires Rust 1.88. `Cargo.lock` pins the exact registry packages that were compiled and
tested. Run `cargo update -p winit-core -p dpi` when intentionally moving to newer releases.

`openharmony-ability` and its derive crate are regular dependencies because this repository is an
OHOS-only backend; they are not hidden behind a `target_env = "ohos"` dependency table.

Applications should depend on the Winit facade only. Its OHOS target dependency selects
`winit-ohos`, so adding this backend a second time would create conflicting Winit core types. An
Ability native-module crate still needs the standard direct N-API/Ability dependencies required by
the `#[ability]` macro expansion:

```toml
[dependencies]
napi-ohos = "1.2"
napi-derive-ohos = "1.2"
openharmony-ability = "1.0.0-beta.2"
winit = { git = "https://github.com/richerfu/winit.git", branch = "master" }

[build-dependencies]
napi-build-ohos = "1.2"
```

## Ability entry

Create the backend event loop inside the `#[ability]` initializer. `run_app` registers callbacks
and returns; it must not block the Ability main thread.

```rust,no_run
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::ohos::{
        EventLoopBuilderExtOpenHarmony,
        ability::{OpenHarmonyApp, ability},
    },
    window::WindowId,
};

#[derive(Default)]
struct App;

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

#[ability]
fn openharmony_app(app: OpenHarmonyApp) {
    EventLoop::builder()
        .with_openharmony_app(app)
        .build()
        .expect("create OHOS event loop")
        .run_app(App)
        .expect("register OHOS event loop");
}
```

The backend translates Ability lifecycle, surface geometry, safe areas, configuration,
XComponent touch/mouse/key input, ArkUI axis/gesture input and IME callbacks into
winit events. `run_app` registers callbacks and returns to the Ability main loop.
`request_redraw` schedules an on-demand NativeVSync tick for each presentable window;
Ability uses `FrameInputDelivery::OnDemand` while NativeVSync is healthy. Creation or
request failure falls back to `Continuous`, which redraws only when requested.
Main-thread wakes are coalesced. Lifecycle and input callbacks request a future frame;
presentation happens after event and data-transfer delivery in a frame iteration.
Background and minimized windows retain redraw demand until they become presentable.
OHOS EGL buffer geometry can lag behind XComponent size callbacks, so size changes
and surface creation redraw every display frame for 200 ms.

`ControlFlow::Poll` follows display refreshes. With no presentable window it behaves
like `Wait` until another platform event arrives. `WaitUntil` uses a timer thread
that sleeps until the selected deadline. `ControlFlow` also determines the next
callback's `StartCause`. The platform owns the loop, so `run_app_on_demand` and
`pump_app_events` are unsupported.

A child surface can be destroyed and recreated without closing its window. The
backend calls `can_create_surfaces` again on recreation; applications must recreate
that window's rendering surface because the old native window handle is invalid.
Native child closure delivers `CloseRequested` followed by `Destroyed` once; dropping
a child from the application delivers only `Destroyed`. Root close requests remain
for the application to decide.

Ability `Destroy` releases rendering surfaces, delivers `Destroyed` to children and
then the root, drops the application handler, and requests process termination.
The `#[ability]` initializer runs only once per process; a destroyed event loop
cannot resume if the system rebuilds the Ability in a cached process.

## Platform capabilities

This workspace carries compatibility patches described in [../README.md](../README.md).
Use its pinned ability and winit revisions together with the matching ArkTS HARs.
`EventLoop` idempotently registers the window, clipboard, URL, files, app-control
and permission facades. `extended-plugins` also registers menus, notifications,
process control, system state and accessibility state. The matching ArkTS factories
must be installed in `NativeAbility.bridgePlugins`; see the complete
[example entry](../../examples/hello_openharmony/arkts/EntryAbility.ets).

Window mutations run asynchronously in per-window order. Native children wait for
surface readiness before their queued configuration runs. Clipboard reads and file
pickers also run asynchronously; never block the Ability UI thread waiting for a
bridge future. Use `task::spawn` and request a repaint when a result arrives.
`events::subscribe` delivers Want, menu, system-state and screen-reader events.
The `file_access` module resolves picker URIs to absolute paths and attempts to persist
and activate grants using `ohos.permission.FILE_ACCESS_PERSIST`. Authorization failures
are logged without discarding valid selections; activation is attempted only for
successfully persisted grants. Native file drags support Copy/Move, owned RGBA previews
and actual OS completion results, retaining their source/data until completion. Pen
pressure, tilt, twist and eraser metadata are forwarded as tablet-tool events. Static
custom cursors and API 22+ 2in1 Ability window icons use typed window-plugin requests.

The [example README](../../examples/hello_openharmony/README.md) documents building
and installing both renderers; [validation](../../examples/hello_openharmony/VALIDATION.md)
records device coverage and remaining platform limitations.

## License

[Apache-2.0](./LICENSE-APACHE)/[MIT](./LICENSE-MIT)

# egui on OpenHarmony

This example runs the egui demos through `eframe`, winit 0.31.0-beta.3, the independent
`winit-ohos` backend, and `openharmony-ability` 1.0.0-beta.2. It supports Glow (EGL/OpenGL ES) and WGPU.

## Build and run the simulator app

On macOS, install DevEco Studio with the HarmonyOS API 26 SDK, its Node/ohpm tools,
and an ARM64 phone, tablet or 2in1 emulator image. The checked-in application declares
all three device types and enables automatic rotation.

```sh
rustup target add aarch64-unknown-linux-ohos
# Set this to your DevEco Studio installation:
export DEVECO_HOME="${DEVECO_HOME:-/Applications/DevEco-Studio.app/Contents}"
./examples/hello_openharmony/build-hap.sh
# Or build the WGPU renderer:
./examples/hello_openharmony/build-hap.sh --features wgpu

export PATH="$DEVECO_HOME/sdk/default/openharmony/toolchains:$PATH"
hdc list targets
hdc -t <device-id> install -r target/hello-openharmony-hap/entry/build/default/outputs/default/entry-default-unsigned.hap
hdc -t <device-id> shell aa start -a EntryAbility -b org.egui.hello
```

`build-hap.sh` builds the native library with `--locked`, uses the vendored Ability
revision and patches below, and assembles the ArkTS runtime and app under `target/`. Generated
dependencies, native binaries and build outputs stay outside the source tree.
The unsigned HAP was installed directly on the emulators listed in
[VALIDATION.md](VALIDATION.md); physical devices require a suitable signing profile.

## Build the native library

Install Rust 1.92 or newer (the workspace's declared minimum) and an OpenHarmony
SDK with native headers and libraries. `OHOS_NDK_HOME` must point to the SDK directory
**containing `native/`**, as required by `napi-build-ohos`.

```sh
rustup target add aarch64-unknown-linux-ohos
export OHOS_NDK_HOME=/path/to/openharmony
./examples/hello_openharmony/build-ohos.sh
```

The script builds with `Cargo.lock`, configures the SDK's clang and sysroot, and writes:

```text
target/aarch64-unknown-linux-ohos/release/libhello_openharmony.so
```

Copy it into the Harmony application's `entry/libs/arm64-v8a/`. Configure the
application's native library dependency as required by your DevEco project. `ohrs`
can also package this cdylib; it must use this workspace's patches and lockfile.

## Match the ArkTS runtime

Use the locally patched `vendor/openharmony-ability/native_ability` and matching plugins, based on
[`richerfu/openharmony-ability` at `06e041fa81862410bee03b7856dc09a489f49639`](https://github.com/richerfu/openharmony-ability/tree/06e041fa81862410bee03b7856dc09a489f49639).
This is the latest runtime-fixes revision used for this adaptation, following
[gpui-ohos `feat/pr82-ability-adaptation`](https://github.com/ohos-rs/gpui-ohos/tree/feat/pr82-ability-adaptation).
The build script uses the same patched source tree for Rust and ArkTS. A package version alone
is insufficient because the branch includes fixes beyond the published beta.

Copy [EntryAbility.ets](arkts/EntryAbility.ets), its matching dependencies and module
configuration when integrating into another HAP. It registers the window, clipboard,
URL, files, app control, permission, menu, notification, process, system state and
accessibility plugins. Keep `defaultPage = true`: ability's default page also hosts
native menus. The example extends `WindowPlugin` at `ui-context-ready` to use
`KeyboardAvoidMode.RESIZE`, so the keyboard resizes the egui canvas instead of
panning the whole XComponent off screen. Preserve this policy when reusing the
entry. Lifecycle overrides must call their `super` implementation.
The page-profile fallback is only needed by the HAP's static page declaration.

The example targets API 26 and requires API 22 (cursor lock APIs). Pasteboard access
requires the manifest permission, runtime consent and a signing profile that allows
`READ_PASTEBOARD`. Cursor lock also requires its declared permission. Phone/tablet
window management follows device policy; the main window's titlebar is OS-owned.

## Runtime behavior

`#[ability]` creates the event loop and `run_native` registers an owned callback,
then returns to ArkTS. GPU surfaces live between `can_create_surfaces` and
`destroy_surfaces`. Native child viewports are created asynchronously, each with
independent rendering, input, IME, accessibility and close handling. The example
limits child dimensions to the monitor so its native controls fit phone screens. Pending children
retain texture uploads until their surface exists. Surface destruction releases GPU
and accessibility resources before the native XComponent is freed.

Touch, mouse, wheel, keyboard, gestures, safe areas and configuration changes feed
egui input. IME commits, preedit, UTF-16 selection/deletion, surrounding text, password
and multiline hints use the native editor. The example loads a CJK fallback font
from the device. Clipboard text and images use the system pasteboard; reads and
permission prompts are asynchronous, and delayed pastes check the focused editor.
URL commands open the system handler. Incoming file drags become egui dropped files;
`winit_ohos::drag_drop::start_file_drag` and winit's `start_drag` support native file
Copy/Move drags, owned RGBA previews and the selected completion action. Cancellation
is distinct from a successful drop. File URIs are decoded through the OHOS file URI API.

If the input service is still starting, IME attachment retries while focused,
with 250/500/1000 ms delays followed by a 2 s maximum backoff. Leaving the editor or losing focus cancels pending retries.
Mouse hover exit cannot cancel an active finger, including the first touch after
a native file drag.

`winit_ohos::file_access::persist_paths` persists and activates picker grants before
returning native paths for Rust file I/O, following GPUI's file access flow. Declare
`ohos.permission.FILE_ACCESS_PERSIST` to use it; permission failures are returned to
the caller. The fileshare facade also exposes querying and revoking grants. The
example saves its text, reads up to 4 KiB from an opened UTF-8 file, and lists a
selected folder. Only one picker is opened at a time; cancellation is reported.

The `extended-plugins` feature exposes typed native menus, notifications, process
control, system state and accessibility state. Subscribe to `winit_ohos::events`
for menu selections, screen-reader changes, display/thermal changes and launch or
new-Want data. These events wake and redraw the application. Register URL schemes
in the HAP manifest when an application needs external deep-link resolution.

The **OHOS capabilities** panel exercises system clipboard, file/folder dialogs,
URLs, native prompts, child windows and window states. **More system capabilities**
adds menus, notifications, system state, restart/close, file dragging and framebuffer
screenshots. These facilities are exposed separately from egui's portable widgets.

`run_and_return` and event pumping are unavailable on OHOS. The SDK does not provide
all desktop operations: arbitrary pointer warping, exclusive video modes and
some main-window decoration changes have no equivalent in this integration.
Static RGBA custom cursors are supported. Runtime application window/task icons
use the API 22+ Ability icon API on 2in1/PC; the OS rejects them on phone/tablet.
Pen pressure, tilt, twist and eraser type flow through winit tablet-tool events
and egui pen metadata. See [PEN-TEST.md](PEN-TEST.md) for native UITest pressure
regression, distinct from the touch-like `uinput -S` command. AccessKit uses an OHOS native provider,
which the OS only queries when an accessibility client is enabled.

See [VALIDATION.md](VALIDATION.md) for actual emulator evidence and remaining limits;
compilation support is distinct from runtime coverage. Physical hardware, physical
touchpad/stylus input and external displays require separate device testing.
Dependency origins and local patches are recorded in
[vendor/README.md](../../vendor/README.md).

# OpenHarmony validation

Regression recorded on 2026-10-09/10 across phone, tablet and 2in1 emulators.
Full capability acceptance remains incomplete: phone folder authorization, 2in1
screen-reader availability and synthetic reader gestures are unresolved. The
expanded suite below is separate from the earlier single-window Glow baseline.

## Earlier revisions and builds

- Mainline rebase: `origin/master` at `62fbf166d96d3c9bc9381f41ca9ed1a437e4f2dc`.
- Winit: `richerfu/winit` `c9ee2bff8a79aacb8068546e7150197693d35a5a` (0.31 beta.3).
- Ability and ArkTS: `2b5defdb0a016b841736470f9c32f2929380c448`, the then-current
  `feat/ohos-adaptation-runtime-fixes` branch, following GPUI's
  `feat/pr82-ability-adaptation` branch.
- Remote revisions above were rechecked at 03:40 on 2026-10-10.
- Both Glow and WGPU HAPs build and install on ARM64 emulators. WGPU uses the
  emulator's available GLES backend; this does not establish Vulkan coverage.

The local, ignored `target/ohos-validation/full/` directory holds screenshots,
logs and the final `glow-final.hap` / `wgpu-final.hap` packages. These add only
the phone child-window size correction to the `*-close.hap` regression packages;
the preferred child size remains unchanged on tablet and 2in1. Superseded HAPs
were removed after recording hashes in `hap-archive.json` (319 MiB reclaimed).
The obsolete 2025 example `dist/` artifacts were also removed (171 MiB), with
hashes in `obsolete-dist-archive.json`.
`target/ohos-validation/results.json` describes only the earlier
ability `8ddea98` baseline and is not the result of the expanded suite.

Final HAP SHA-256:

- Glow: `f37834ea076d06d69ffebd1192440f3d89ed5a0604baf2eb7103581e302b9700`
- WGPU: `b742a64ae641cc4699d77701fe051a6be70d639d44c9d4d87bbae9f7b630e007`

## Devices

| Type | Instance | Image | Resolution |
| --- | --- | --- | --- |
| Phone | Arkdown_API26 / Pura 90 Pro | API 26, 7.0.0.106 | 1256 × 2760 |
| Tablet | Egui_Tablet_API26 / MatePad Pro 11 | API 26, 7.0.0.107 | 2560 × 1600 |
| 2in1 | MateBook Pro | API 26, 7.0.0.106 | 3120 × 2080 |

HDC ports change after emulator restarts. Check `param get const.product.devicetype`
and the emulator instance log before interacting with a device. Run one emulator
at a time when investigating startup or input-method failures.

## Observed results

The earlier Glow baseline passed installation/cold launch, demo rendering,
checkbox/slider input, scrolling, two-finger gestures, English/Chinese input,
delete/Enter, Home/resume and cold restart on all three types. Phone/tablet rotation
and 2in1 mouse, wheel, Shift, Ctrl+A/C/V, move/resize/maximize/minimize also passed.
These observations remain useful regression evidence, but do not cover later native
clipboard, child-window, accessibility or file integration changes.

Expanded suite, recorded separately:

| Capability | Observed result and evidence |
| --- | --- |
| Phone WGPU IME | Chinese candidate commit, selected-text replacement, deletion and consecutive English input resulted in `你abc`; password masking and independent native-child input passed. `phone-ime-commit.jpeg`, `phone-password.jpeg`, `phone-child-ime.jpeg`. |
| Phone WGPU lifecycle | Home/resume retained PID 13403 and text; portrait/landscape redraw passed. `phone-before-home.jpeg`, `phone-wgpu-rotated.jpeg`. |
| Phone system clipboard | Native text readback `你abc`; 32 × 32 RGBA image read back as 137-byte PNG. `phone-wgpu-regression.log`. |
| Phone native menus/notifications | Menu selection returned `hello`; permission, notification publication and dismissal passed. `phone-notify-allowed.jpeg`, `phone-wgpu-regression.log`. |
| Phone WGPU screenshot | Screenshot event returned 1256 × 2526 pixels. `phone-wgpu-regression.log`. |
| Phone new Want | URI `egui-test://regression/phone` and string parameter `device=phone` reached the application. `phone-wgpu-regression.log`. |
| Phone file save/read | Picker grants were persisted and activated; Rust wrote and read back the 17-byte `Hello 鸿蒙 😀` file. `phone-files-regression.log`, `phone-save-io-confirm.jpeg`, `phone-read-selected.jpeg`. |
| Phone Glow native drag | Root and child received actual files; subsequent child creation/close and system dialog passed. `phone-glow-root-child-final.log` at 01:55:16 and 01:56:38; `phone-glow-root-drag3.jpeg`. |
| Phone accessibility | Native provider node queries and native click action reached egui and opened a system dialog. `phone-a11y-native-action.log`, `phone-a11y-native-action.jpeg`. Screen-reader state events also passed. |
| 2in1 WGPU drag/drop | Repeated native drags delivered an actual file to both root and native-child viewports. `2in1-drag-fixed.log` at 00:43:50 and 00:44:44. |
| 2in1 WGPU process control | Restart changed the process; Close terminated it; subsequent launch rendered. `2in1-restart-close.log` and screenshots. |
| 2in1 Glow | Native child rendering/close, framebuffer screenshot 3120 × 1885 and PNG clipboard readback passed. `2in1-glow-final.log` and child screenshots. |
| Tablet file I/O | WGPU saved and read back `TAB`, then selected Documents and enumerated its entries. `tablet-wgpu-final.log`, `tablet-readback.jpeg`, `tablet-folder-readback.jpeg`. |
| Tablet IME | WGPU committed `你好`, masked three password characters and edited a native child. `KeyboardAvoidMode.RESIZE` keeps the root editor visible. First focus also opened the keyboard in the retry build. `tablet-ime-resize-pinyin.jpeg`, `tablet-password-done.jpeg`, `tablet-retry-first-focus.jpeg`. |
| Tablet expanded system suite | Text/image clipboard, menu `hello`, notification/dismiss, display/thermal state and framebuffer screenshot (2560 × 1450) passed. `tablet-wgpu-retry.log`. |
| Tablet native drag and lifecycle | Actual root/child file drops, first-touch child creation/close after drag, dialog accept/cancel, Home/resume, Want parameters and portrait/landscape redraw passed. `tablet-touch-handoff.log`, `tablet-touch-handoff-child.jpeg`, `tablet-wgpu-portrait-final.jpeg`. |
| Tablet URL | Browser opened `egui.rs`; the remote web demo reports unavailable WebGL2 in the emulator browser. Native URL handoff passed; this is not web-renderer validation. `tablet-browser-ready.jpeg`. |
| Tablet accessibility | Reader state, individual provider node labels/bounds and native click action opening a system dialog passed. `tablet-a11y-nodes.txt`, `tablet-a11y-click-10.jpeg`. |
| Tablet final Glow input/drag | Chinese commit/readback `你好`, masked password, independent child soft-keyboard input/delete/Done, actual root and child file drops, and the first touch after each drop passed. `tablet-glow-chinese-final.jpeg`, `tablet-glow-password-dismiss.jpeg`, `tablet-glow-child-edited.jpeg`, `tablet-glow-drag-child.jpeg`, `tablet-glow-final.log`. |
| Tablet final Glow file I/O | Saved 6 UTF-8 bytes, then reopened and read `你好` from `egui-1791572071321.txt`. `tablet-glow-saved.jpeg`, `tablet-glow-folder.jpeg`. |
| Tablet final Glow Close | The Close button terminated PID 9613 after EGL teardown and returned to the previous application. `tablet-glow-close-confirmed.jpeg`, `tablet-glow-close-init.log`. |
| Tablet final Glow system/lifecycle | Folder enumeration, PNG clipboard (137 bytes), dialog accept/cancel, menu selection, notification/dismiss, system state, screenshot (2560 × 1450), Want URI/parameters and both rotations passed. Restart changed PID 12089 → 24303; Close then removed the process. `tablet-glow-final.log`, `tablet-glow-portrait.jpeg`, `tablet-glow-restarted.jpeg`, `tablet-glow-final-exit.jpeg`. |
| 2in1 final WGPU input/drag | Maximization, Shift uppercase, Ctrl+A/C/V (`AbcAbc`), Chinese candidate commit (`你好`), masked password, child editing/deleting an emoji, actual root/child file drops and first-touch follow-up passed. `2in1-wgpu-shortcuts.jpeg`, `2in1-wgpu-chinese.jpeg`, `2in1-wgpu-child-input.jpeg`, `2in1-wgpu-drag-child.jpeg`, `2in1-wgpu-final.log`. |
| 2in1 final WGPU file I/O | Saved 6 UTF-8 bytes in Documents and read back `你好` from `egui-1791572762112.txt`. `2in1-wgpu-saved.jpeg`, `2in1-wgpu-folder.jpeg`. |
| 2in1 final WGPU system/window | Folder enumeration (4 entries), PNG clipboard (137 bytes), dialog accept/cancel, notification/dismiss, menu `hello`, display/thermal state, screenshot (3120 × 1885), Want URI/parameters, fullscreen/windowed, move/resize and minimize/resume passed. `2in1-wgpu-final.log`, `2in1-wgpu-fullscreen.jpeg`, `2in1-wgpu-move-resize.jpeg`, `2in1-wgpu-resume.jpeg`. |
| 2in1 final WGPU process/input | Restart changed PID 3378 → 16985; Close terminated it. Consecutive Shift/Latin key input at 100 ms intervals produced `Abc`. A zero-delay synthetic batch did not preserve all letters, so it is not counted as a pass. `2in1-wgpu-final-restart.jpeg`, `2in1-wgpu-final-close.jpeg`, `2in1-wgpu-close-final.log`, `2in1-wgpu-fast-typing.jpeg`. |
| 2in1 final Glow | Latin root/child input, whole-emoji deletion, text/PNG clipboard, actual root/child drops and immediate touch, dialog, notification/dismiss, system state, framebuffer screenshot, minimize/resume and Want data passed. Restart changed PID 18333 → 19566; Close then terminated it (`2in1-glow-exit-final.log`). `2in1-glow-close-final.log`, `2in1-glow-child-final.jpeg`, `2in1-glow-drag-child-final.jpeg`, `2in1-glow-system-final.jpeg`. |
| Phone final WGPU input | First tap opened the soft keyboard; Chinese commit/readback `你好`, three masked password characters, child soft-keyboard edit/delete/Done and root Close (PID 3270) passed. `phone-wgpu-final-first-focus.jpeg`, `phone-wgpu-final-chinese.jpeg`, `phone-wgpu-final-child-edit.jpeg`, `phone-wgpu-close-final.log`. |
| Phone final WGPU size/system/lifecycle | Native child controls fit the screen and native X closes it. Actual root/child drops followed by immediate touch, PNG clipboard, dialog, notification/dismiss, display/thermal state, screenshot, saving/reading 17-byte `Hello 鸿蒙 😀`, Home/resume, Want and both rotations passed. Restart changed PID 12890 → 15550; Close terminated it. `phone-wgpu-sized-final.log`, `phone-wgpu-sized-child.jpeg`, `phone-wgpu-final-child-handoff-correct.jpeg`, `phone-wgpu-final-file-confirm.jpeg`, `phone-wgpu-final-resume.jpeg`, `phone-wgpu-sized-exit.log`. |
| Phone final Glow | First-focus soft keyboard, Chinese commit/readback `你好`, three masked password characters, child soft-keyboard edit/delete/Done, screen-fitting child size, actual root/child drops and first-touch follow-up passed. PNG clipboard, dialog, notification/dismiss, state, screenshot and Home/resume/Want passed. Restart changed PID 17944 → 20796; Close terminated it. `phone-glow-sized-final.log`, `phone-glow-final-child.jpeg`, `phone-glow-final-child-handoff.jpeg`, `phone-glow-final-exit.log`. |
| Tablet final WGPU Close | Final WGPU package rendered, then its Close button terminated PID 27505. `tablet-wgpu-close-launch.jpeg`, `tablet-wgpu-close-confirmed.jpeg`, `tablet-wgpu-close-final.log`. |

## Fixes and remaining limitations

- A native drag reentered XComponent's callback and crashed; startup now dispatches
  through a separate main-thread task. A second hang came from taking egui's context
  lock while rendering the dropped-file label; the sample now releases the input
  lock first. Subsequent WGPU root/child drops passed.
- Outgoing native drag completion now clears held mouse buttons and cancels active
  touches. Regression tests confirm that the next mouse click and new finger work.
  A phone Glow run froze in the emulator driver `d_eglSwapBuffers_special` after a
  drop (`phone-drag-release-freeze.log`, 01:53). A subsequent process completed
  root/child drops and continued interacting; this intermittent failure remains recorded.
- Switching mouse to touch exposed a late native mouse-leave between touch down/up.
  It cleared the emulated finger position and lost the release. The integration now
  preserves an active finger across mouse leave. A regression test and tablet native
  root/child drag followed immediately by touch verify the fix.
- Cold startup under concurrent emulator load generated freeze reports in native
  library loading (2in1) and emulator EGL context creation (tablet); the applications
  later recovered. See `2in1-startup-freeze.log` and `tablet-wgpu-startup-freeze.log`.
  It would be incorrect to claim that this entire validation run had no freezes.
- After emulator restarts, IME attach returned 12800008 on phone/tablet; native
  Settings text input was also affected. Phone recovered when run alone and passed
  the input flows above. The earlier bounded 250/500/1000 ms retry handled startup failures
  while the editor remains focused. Tablet first-focus and root/child editing passed;
  final phone Glow and WGPU first focus also opened the keyboard immediately.
  Tablet Glow later reproduced
  12800008 on first focus and recovered on another tap (`tablet-glow-final.log`);
  automatic recovery from every cold-service failure is not established.
- Phone folder mode filters folders but currently exposes no working selection
  confirmation in this emulator. Directory readback is not yet verified. Huawei’s
  [current picker reference](https://developer.huawei.com/consumer/en/doc/harmonyos-references/js-apis-file-picker)
  lists phone folder mode from API 26; this observation is an image-specific
  validation failure, not evidence that all phones lack the API. A final repeat
  confirmed the missing completion UI after entering My phone, including long
  press; cancellation returned normally (`phone-folder-browse-final.jpeg`,
  `phone-folder-long-final.jpeg`, `phone-wgpu-close-final.log`).
- The 2in1 API 26 image has no installed screen-reader bundle or corresponding
  Settings entry. Its inactive XComponent provider reports `childTree: -1`; reader
  interaction cannot be verified on that image. `2in1-installed-bundles.txt`,
  `2in1-settings-scroll2.jpeg`, `2in1-final-inspector.txt`.
- Recursive native accessibility tree dumping hung in the platform dumper. Individual
  provider node queries and action execution passed. Automated screen-reader double
  taps did not activate either egui or the native dialog; that gesture remains
  unverified. Screen reader was restored to off after the test.
- OHOS root Close released egui resources but waited for a desktop-style Destroyed
  event that Ability never emits for a dropped Rust window. Eframe now exits its
  loop after resource teardown; final tablet Glow and WGPU termination passed.
  Final 2in1 and phone Glow/WGPU termination also passed.
- Serial final phone and 2in1 runs added no app crash/freeze reports. This does not
  erase earlier intermittent emulator driver/service failures. Native accessibility
  was exercised on phone/tablet; 2in1 requires an image with a screen-reader service.

## Build and static checks

Workspace/all-target/all-feature checking and Clippy, OHOS WGPU Clippy, Linux and
Windows eframe cross-checks, WebAssembly eframe cross-check, documentation, formatting
and repository lint passed. Eleven egui-winit regression tests passed, covering IME,
Unicode selection/deletion, touch cancellation, release after outgoing native drags and asynchronous file drops. The web
check caught and fixed a `Cell<IMEOutput>` regression after IMEOutput stopped being
Copy. The new file-access path also passed OHOS Clippy and Glow/WGPU HAP builds.
The final phone child-size correction passed both HAP builds, OHOS WGPU Clippy,
formatting and repository lint; its native close button was exercised on phone.

## Reproduce the expanded suite

Build both renderers using [README.md](README.md). For each device and renderer:

1. Cold launch; exercise the demo's checkbox, slider, scrolling and two-finger view.
2. In **OHOS capabilities**, edit Latin/Chinese/emoji text, select/replace/delete,
   switch to the password field, dismiss and refocus the keyboard. Repeat inside
   **Native child viewport**, then close/reopen it. On 2in1 include physical keys,
   Shift, Ctrl+A/C/V, mouse and wheel.
3. Copy/read text; copy an image and **Inspect clipboard**. Exercise permission
   acceptance and cancellation. Save a text file, reopen and read it; select a
   folder and verify its entries. Cancel each picker once.
4. Open a website and return; accept/cancel a system dialog. Open a native menu and
   select an item; publish/dismiss a notification. Check **System state** and
   **Screenshot** results. Send a new Want URI and check its application event.
5. Drag **Drag sample file** onto the root and child, checking the actual dropped
   path in logs. Repeatedly create/close children. Exercise supported window states.
6. Home/resume with text retained; rotate phone/tablet; resize/maximize/restore and
   minimize 2in1. Test Restart, Close and fresh launch. Check app crash/freeze reports.
7. Enable the screen reader, query individual native provider nodes, execute an
   accessibility click and restore the reader's previous state.

Physical hardware, physical touchpads/styluses, external displays and API versions
below 26 remain untested. Pointer warping, exclusive video modes and some OS-owned
main-window decorations have no supported equivalent. Native phone/tablet window
operations follow OS policy. The cursor, icon, drag and pen updates below supersede
the earlier gaps in those areas.


## Capability update — 2026-10-10/11

Reference GPUI branch: `feat/pr82-ability-adaptation`, `58e6b6ba7760b0a3783a1c2758c1cd812544c954`.
Ability Rust/ArkTS source is now vendored from **`06e041fa81862410bee03b7856dc09a489f49639`**,
with the local extensions listed in [vendor/README.md](../../vendor/README.md).
No phone layout change is included. The previously identified PhotoCraft public
atomic-save failure is explicitly outside this update.

Implemented:

- Static RGBA custom cursors with validated dimensions, byte counts and hotspots.
- API 22+ Ability window/task icons on 2in1/PC, including restoring the packaged
  icon. Phone/tablet return the OS capability limitation; child windows share the
  Ability icon. PixelMap creation specifies both source and destination RGBA format.
- Native Copy/Move negotiation, preview pixels/hotspots, and owned completion
  results. Cancellation emits `OutgoingDragCanceled`, not a successful drop.
- Pen pressure, tilt, twist and eraser tool type through native bindings, Ability,
  winit tablet-tool events and egui. Corrected the XComponent tool query to use
  the touch-point index rather than its contact ID. Cancellation cannot leave a
  pointer pressed or generate a click.
- IME startup retries continue while focused with 250/500/1000 ms delays then a
  2 s cap. Success, focus loss and leaving the editor reset the retry state.

Observed in this update:

| Check | Result |
| --- | --- |
| Phone WGPU native pen | UITest `penSwipe` / `injectPenPointerAction` produced egui Start/Move/End at 0.25, 0.5, 0.75 and 1.0 pressure. `phone-uitest-pen.log`. |
| Tablet WGPU native pen | Same four pressures and lifecycle phases passed. `tablet-uitest-pen.log`. |
| Tablet Glow native pen | Same four pressures and lifecycle phases passed in a separate app process. `tablet-glow-uitest-pen.log`. |
| 2in1 WGPU / Glow native pen | Both renderers passed all four pressures and lifecycle phases in fresh test processes (4125 / 13122). `2in1-uitest-pen.log`, `2in1-glow-uitest-pen.log`, with matching successful `*-run.log` files. |
| 2in1 drag completion | Native accepted drop returned Copy; pressing Esc while dragging returned Cancelled. `capability-2in1-drag-icon.log`, `capability-2in1-drag-escape.log`. |
| 2in1 custom cursor/icon | Native plugin calls returned success; the latest WGPU package visibly displays the correct orange runtime title icon after the source RGBA fix. `2in1-rgba-icon-cursor.log`, `Screenshot_2026-10-11T002537.png`. |
| PhotoCraft tablet | Native pen pressure changes brush width; native touch opens Type editing, soft-keyboard pinyin commits `你好`, clipboard pastes CJK/emoji, Backspace removes a whole emoji; native child rendering/close and Home/resume passed. See its packaging validation report. |

[`PEN-TEST.md`](PEN-TEST.md) contains the reproducible **native** pen test. Official
`uinput -S` is documented as touch-equivalent and did report Finger/zero pressure;
this is not a lack of simulator pen support. UITest provides real pen type and
pressure injection. Its public pen APIs do not expose nonzero tilt, barrel twist
or eraser selection, so these properties are unit-tested but not device-verified.
Move completion mapping is implemented; the observed file-open targets selected
Copy, so a native Move drop is not claimed as tested.

Validation: egui-winit **13 tests passed**; workspace/all-target/all-feature check,
formatting and host egui-winit strict Clippy passed. OHOS WGPU Clippy passed with
only the pre-existing large embedded-font lint allowed. Both Glow and WGPU HAPs
built successfully. The Ability ArkTS runtime suite passed **15 tests**. Standalone
Ability Rust unit tests cannot link on macOS (`native_window` is an OHOS library);
the platform code was cross-compiled and the device paths above exercised.

Latest artifacts use the `capability-*` prefix under `target/ohos-validation/full/`;
the hashes earlier in this document refer to the previous validation packages.

Latest SHA256:

| Package | SHA256 |
| --- | --- |
| WGPU | `274c9824e3c177b8e8ff2734857c1ac137bb49f6bc5a65b338610d2dcb96b208` |
| Glow | `2b62e9f03aa41c5a0a9978cf58b01e816216c5cca42e13f397731a1bc908c241` |
| UITest module | `47fe34b9c412d3b19e831c36c6d8a0fe6b36981b6038533a2f882dc5f31ab375` |

## Branch snapshot checks — 2026-10-11

The adaptation was collected on `ohos`, based on mainline
`62fbf166d96d3c9bc9381f41ca9ed1a437e4f2dc`. The source snapshot retains the
matched vendored Rust/ArkTS runtime and the dependency revisions documented above.

Fresh checks passed: formatting, repository lint, locked workspace/all-target/
all-feature checking, 13 egui-winit unit tests, host workspace Clippy, and OHOS
Glow/WGPU Clippy. Clippy uses `-D warnings -A clippy::large_include_file` because
the base branch already embeds the large HarmonyOS font. The build scripts passed
shell syntax checking. Logs are under `target/ohos-validation/branch-ohos/`.

The pen log verifier also passed against the five archived phone, tablet and
2in1 UITest runs listed above. This rechecks the recorded evidence; it is not a
new round of emulator interaction.

Separately, the native binding repository added API 24 owned pointer-event APIs
in [d4ac216](https://github.com/ohos-rs/ohos-native-bindings/commit/d4ac216f498cd7504a8fc860758e3fe5ecd6a899).
Its 2in1 application probe passed 33 setter calls, pressure readback and clone/drop,
but newly created events failed nonzero tilt and rotation readback because native
getters select different fields. That binding-only probe is not part of this
egui snapshot's dependency set or end-to-end pen acceptance.

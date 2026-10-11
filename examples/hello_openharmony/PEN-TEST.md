# Native pen regression with UITest

`hdc shell uinput -S` has a stylus name, but the [official uinput
manual](https://github.com/openharmony/docs/blob/master/zh-cn/application-dev/dfx/uinput.md)
states that its effect is equivalent to touch input. On the API 26 emulator it
arrives at XComponent as Finger with pressure 0. It cannot validate pen pressure.

Use the [UITest Driver pen APIs (API 18+)](https://github.com/openharmony/docs/blob/master/zh-cn/application-dev/reference/apis-test-kit/js-apis-uitest.md#penswipe18)
instead. `penSwipe` supports a pressure argument in [0, 1];
`injectPenPointerAction` supports a multi-point path with constant pressure.
The separate test runner exercises 0.25, 0.75, 1.0 and 0.5. Events travel through
the OS input service, XComponent, Ability, winit and egui, rather than inserting
events directly into egui.

```sh
./examples/hello_openharmony/build-pen-test.sh --features wgpu
# Select a connected device and a clear area inside the demo's root window.
hdc -t DEVICE install -r target/hello-openharmony-hap/entry/build/default/outputs/default/entry-default-unsigned.hap
hdc -t DEVICE install -r target/hello-openharmony-hap/entry/build/default/outputs/ohosTest/entry-ohosTest-unsigned.hap
# The SDK imports UITest before the custom runner has an AbilityDelegator.
# Pre-starting its documented daemon supplies the default connection token.
hdc -t DEVICE shell uitest start-daemon default
hdc -t DEVICE shell aa test -b org.egui.hello -m entry_test \
  -s unittest OpenHarmonyTestRunner -s x 1000 -s y 1600 -w 30 > pen-run.log
hdc -t DEVICE shell hilog -x > pen.log
python3 examples/hello_openharmony/verify-pen-log.py pen.log pen-run.log
```

The runner returning 0 only establishes that injection succeeded. The log check
requires Start, Move and End with each injected pressure at the egui boundary,
and a successful result in the matching runner output. Failed injector startup
must not be mistaken for success by reading events from an older run.
Use a fresh app process/log capture to avoid accepting stale test records.
The runner also accepts `-s bundle BUNDLE` to drive another foreground app, but
that app needs its own assertions (the log checker targets this demo). Add
`-s pressure 0.25` to inject just one stroke when comparing brush output.

No tested public UITest pen method accepts tilt, barrel rotation or eraser tool
selection. Zero tilt/twist from these injected events is not validation of those
properties. Their conversion, cancellation and eraser mapping have unit tests;
nonzero hardware values still need a physical pen or a suitable native injector.
Do not interpret a synthetic pointer inside PhotoCraft's control channel as an
OS pen-input test. The ordinary HAP does not contain this test runner.

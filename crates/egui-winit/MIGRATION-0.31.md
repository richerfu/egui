# Migrating this fork to winit 0.31

- Windows are `Box<dyn Window>` or `Arc<dyn Window>`; callback event loops are
  `&dyn ActiveEventLoop`. Pass `window.as_ref()` into egui integration methods.
- `EventLoop` is no longer generic. Eframe keeps typed repaint and accessibility
  events in a channel and uses `EventLoopProxy::wake_up` to drain it. External loops
  must forward `proxy_wake_up` to `EframeWinitApplication`.
- Initialize surfaces in `can_create_surfaces`, release them in `destroy_surfaces`,
  and clean up on application drop. Forward both callbacks when embedding eframe.
- `run_app` owns a `'static` application. `run_native` therefore takes
  `AppCreator<'static>`. `create_native` still permits borrowed state when used with
  `run_app_on_demand` or `pump_eframe_app` on supported desktop platforms.
- Pass the active event loop to `egui_winit::State::on_window_event` and
  `egui_glow::EguiGlow::on_window_event`. It is needed for asynchronous file drops.
- Winit surface size APIs replace inner-size APIs. Egui's `ViewportBuilder` still
  uses `with_inner_size`; its units and behavior are unchanged. Surface position is
  relative to the outer window, so screen coordinates include both offsets.
- Winit mouse and touch input use Pointer events. IME uses explicit capabilities;
  surrounding-text deletion is not requested because egui does not expose the
  required byte-indexed text editing interface. On OHOS the native extension supplies
  UTF-16 surrounding text, selection and deletion directly to TextEdit.
- `IMEOutput` is now Clone rather than Copy, and carries surrounding text and input
  hints. Password editors publish an empty surrounding-text snapshot.

See the external event loop examples and `egui_glow/examples/pure_glow.rs` for full
applications, and `examples/hello_openharmony` for the callback-driven OHOS entry.

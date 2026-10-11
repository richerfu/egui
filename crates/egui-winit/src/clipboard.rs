#[cfg(all(target_env = "ohos", feature = "clipboard"))]
mod ohos;

use raw_window_handle::RawDisplayHandle;

/// Handles interfacing with the OS clipboard.
///
/// If the "clipboard" feature is off, or we cannot connect to the OS clipboard,
/// then a fallback clipboard that just works within the same app is used instead.
pub struct Clipboard {
    #[cfg(all(target_env = "ohos", feature = "clipboard"))]
    ohos: Option<ohos::Clipboard>,
    #[cfg(all(
        not(any(target_os = "android", target_os = "ios", target_env = "ohos")),
        feature = "arboard",
    ))]
    arboard: Option<arboard::Clipboard>,

    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        not(target_env = "ohos"),
        feature = "smithay-clipboard"
    ))]
    smithay: Option<smithay_clipboard::Clipboard>,

    /// Fallback manual clipboard.
    clipboard: String,
}

impl Clipboard {
    /// Construct a new instance
    pub fn new(_raw_display_handle: Option<RawDisplayHandle>) -> Self {
        Self {
            #[cfg(all(target_env = "ohos", feature = "clipboard"))]
            ohos: None,
            #[cfg(all(
                not(any(target_os = "android", target_os = "ios", target_env = "ohos")),
                feature = "arboard",
            ))]
            arboard: init_arboard(),

            #[cfg(all(
                any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd"
                ),
                not(target_env = "ohos"),
                feature = "smithay-clipboard"
            ))]
            smithay: init_smithay_clipboard(_raw_display_handle),

            clipboard: Default::default(),
        }
    }

    #[cfg(all(target_env = "ohos", feature = "clipboard"))]
    pub(crate) fn init_ohos(&mut self, app: &winit::platform::ohos::ability::OpenHarmonyApp) {
        self.ohos
            .get_or_insert_with(|| ohos::Clipboard::new(app.clone()));
    }

    pub(crate) fn request_paste(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        #[cfg(all(target_env = "ohos", feature = "clipboard"))]
        if let Some(clipboard) = &self.ohos {
            clipboard.request_paste(ctx);
            return;
        }
        let _ = ctx;
        if let Some(text) = self.get().filter(|text| !text.is_empty()) {
            input
                .events
                .push(egui::Event::Paste(text.replace("\r\n", "\n")));
        }
    }

    #[cfg(all(target_env = "ohos", feature = "clipboard"))]
    pub(crate) fn drain_pastes(&self, ctx: &egui::Context, input: &mut egui::RawInput) {
        if let Some(clipboard) = &self.ohos {
            clipboard.drain_pastes(ctx, input);
        }
    }

    #[cfg_attr(
        target_env = "ohos",
        expect(
            clippy::needless_pass_by_ref_mut,
            reason = "other platform clipboard implementations require mutable access"
        )
    )]
    pub fn get(&mut self) -> Option<String> {
        #[cfg(all(target_env = "ohos", feature = "clipboard"))]
        if let Some(clipboard) = &self.ohos {
            return clipboard.get();
        }
        #[cfg(all(
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ),
            not(target_env = "ohos"),
            feature = "smithay-clipboard"
        ))]
        if let Some(clipboard) = &mut self.smithay {
            return match clipboard.load() {
                Ok(text) => Some(text),
                Err(err) => {
                    log::error!("smithay paste error: {err}");
                    None
                }
            };
        }

        #[cfg(all(
            not(any(target_os = "android", target_os = "ios", target_env = "ohos")),
            feature = "arboard",
        ))]
        if let Some(clipboard) = &mut self.arboard {
            return match clipboard.get_text() {
                Ok(text) => Some(text),
                Err(err) => {
                    log::error!("arboard paste error: {err}");
                    None
                }
            };
        }

        Some(self.clipboard.clone())
    }

    pub fn set_text(&mut self, text: String) {
        #[cfg(all(target_env = "ohos", feature = "clipboard"))]
        if let Some(clipboard) = &self.ohos {
            clipboard.set_text(text);
            return;
        }
        #[cfg(all(
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ),
            not(target_env = "ohos"),
            feature = "smithay-clipboard"
        ))]
        if let Some(clipboard) = &mut self.smithay {
            clipboard.store(text);
            return;
        }

        #[cfg(all(
            not(any(target_os = "android", target_os = "ios", target_env = "ohos")),
            feature = "arboard",
        ))]
        if let Some(clipboard) = &mut self.arboard {
            if let Err(err) = clipboard.set_text(text) {
                log::error!("arboard copy/cut error: {err}");
            }
            return;
        }

        self.clipboard = text;
    }

    #[cfg_attr(
        target_env = "ohos",
        expect(
            clippy::needless_pass_by_ref_mut,
            reason = "other platform clipboard implementations require mutable access"
        )
    )]
    pub fn set_image(&mut self, image: &egui::ColorImage) {
        #[cfg(all(target_env = "ohos", feature = "clipboard"))]
        if let Some(clipboard) = &self.ohos {
            clipboard.set_image(image);
            return;
        }
        #[cfg(all(
            not(any(target_os = "android", target_os = "ios", target_env = "ohos")),
            feature = "arboard",
        ))]
        if let Some(clipboard) = &mut self.arboard {
            if let Err(err) = clipboard.set_image(arboard::ImageData {
                width: image.width(),
                height: image.height(),
                bytes: std::borrow::Cow::Borrowed(bytemuck::cast_slice(&image.pixels)),
            }) {
                log::error!("arboard copy/cut error: {err}");
            }
            log::debug!("Copied image to clipboard");
            return;
        }

        log::error!(
            "Copying images is not supported. Enable the 'clipboard' feature of `egui-winit` to enable it."
        );
        _ = image;
    }
}

#[cfg(all(
    not(any(target_os = "android", target_os = "ios", target_env = "ohos")),
    feature = "arboard",
))]
fn init_arboard() -> Option<arboard::Clipboard> {
    profiling::function_scope!();

    log::trace!("Initializing arboard clipboard…");
    match arboard::Clipboard::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            log::warn!("Failed to initialize arboard clipboard: {err}");
            None
        }
    }
}

#[cfg(all(
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    not(target_env = "ohos"),
    feature = "smithay-clipboard"
))]
fn init_smithay_clipboard(
    raw_display_handle: Option<RawDisplayHandle>,
) -> Option<smithay_clipboard::Clipboard> {
    #![expect(clippy::undocumented_unsafe_blocks)]

    profiling::function_scope!();

    if let Some(RawDisplayHandle::Wayland(display)) = raw_display_handle {
        log::trace!("Initializing smithay clipboard…");
        #[expect(unsafe_code)]
        Some(unsafe { smithay_clipboard::Clipboard::new(display.display.as_ptr()) })
    } else {
        #[cfg(feature = "wayland")]
        log::debug!("Cannot init smithay clipboard without a Wayland display handle");
        #[cfg(not(feature = "wayland"))]
        log::debug!(
            "Cannot init smithay clipboard: the 'wayland' feature of 'egui-winit' is not enabled"
        );
        None
    }
}

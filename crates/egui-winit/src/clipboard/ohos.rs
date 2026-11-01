//! Ordered, asynchronous access to the system pasteboard.
use egui::mutex::Mutex;
use std::sync::{Arc, mpsc};

use winit::platform::ohos::{
    ability::OpenHarmonyApp,
    plugins::clipboard::{ClipboardBridgePlugin, ClipboardClient},
    plugins::permission::PermissionExt as _,
    task::CommandQueue,
};

pub(super) struct Clipboard {
    app: OpenHarmonyApp,
    commands: CommandQueue,
    cache: Arc<Mutex<Option<(u32, String)>>>,
    paste_tx: mpsc::Sender<(Option<egui::Id>, String)>,
    paste_rx: mpsc::Receiver<(Option<egui::Id>, String)>,
}

impl Clipboard {
    pub fn new(app: OpenHarmonyApp) -> Self {
        let (paste_tx, paste_rx) = mpsc::channel();
        Self {
            app,
            commands: CommandQueue::default(),
            cache: Arc::default(),
            paste_tx,
            paste_rx,
        }
    }

    fn revision(app: &OpenHarmonyApp) -> Option<u32> {
        let plugin = app
            .registered_plugin::<ClipboardBridgePlugin>()
            .ok()
            .flatten()?;
        plugin.revision().ok()
    }

    pub fn get(&self) -> Option<String> {
        let revision = Self::revision(&self.app);
        let value = self
            .cache
            .lock()
            .as_ref()
            .and_then(|(stored, text)| (Some(*stored) == revision).then(|| text.clone()));
        if value.is_none() {
            self.read(None);
        }
        value
    }

    pub fn request_paste(&self, ctx: &egui::Context) {
        self.read(Some(ctx.clone()));
    }

    fn read(&self, ctx: Option<egui::Context>) {
        let app = self.app.clone();
        let cache = Arc::clone(&self.cache);
        let tx = self.paste_tx.clone();
        let target = ctx.as_ref().and_then(|ctx| ctx.memory(|m| m.focused()));
        self.commands.push(async move {
            let revision = Self::revision(&app);
            let result: Result<_, winit::platform::ohos::ability::napi_ohos::Error> = async {
                if ctx.is_some() {
                    let _ = app
                        .request_permission("ohos.permission.READ_PASTEBOARD")
                        .await?;
                }
                Ok(ClipboardClient::new(&app)?.read_text().await?)
            }
            .await;
            match result {
                Ok(text) => {
                    *cache.lock() = revision
                        .filter(|revision| Some(*revision) == Self::revision(&app))
                        .zip(text.clone());
                    if let (Some(ctx), Some(text)) = (ctx, text) {
                        let _ = tx.send((target, text));
                        ctx.request_repaint();
                    }
                }
                Err(error) => log::warn!("OHOS pasteboard read failed: {error}"),
            }
        });
    }

    pub fn drain_pastes(&self, ctx: &egui::Context, input: &mut egui::RawInput) {
        let focused = ctx.memory(|m| m.focused());
        for (target, text) in self.paste_rx.try_iter() {
            // A permission dialog/read can outlive the editor that requested it.
            if target == focused && !text.is_empty() {
                input
                    .events
                    .push(egui::Event::Paste(text.replace("\r\n", "\n")));
            }
        }
    }

    pub fn set_text(&self, text: String) {
        let app = self.app.clone();
        let cache = Arc::clone(&self.cache);
        self.commands.push(async move {
            let result = async { ClipboardClient::new(&app)?.write_text(text.clone()).await }.await;
            match result {
                Ok(()) => {
                    *cache.lock() = Self::revision(&app).map(|revision| (revision, text));
                }
                Err(error) => log::warn!("OHOS pasteboard write failed: {error}"),
            }
        });
    }

    pub fn set_image(&self, image: &egui::ColorImage) {
        let app = self.app.clone();
        let width = image.width() as u32;
        let height = image.height() as u32;
        let rgba: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(|pixel| pixel.to_srgba_unmultiplied())
            .collect();
        self.commands.push(async move {
            let result = async {
                ClipboardClient::new(&app)?
                    .write_image(&rgba, width, height)
                    .await
            }
            .await;
            if let Err(error) = result {
                log::warn!("OHOS image clipboard write failed: {error}");
            }
        });
    }
}

//! Interactive examples of the platform APIs used by the integration.
use eframe::egui;
use egui::mutex::Mutex;
use std::sync::Arc;
use winit::platform::ohos::{
    ability::OpenHarmonyApp,
    plugins::{
        clipboard::ClipboardClient,
        files::{FileDialogOptions, FilesExt as _},
        permission::PermissionExt as _,
        window::{WindowClient, WindowPromptRequest},
    },
    task,
};

pub struct Capabilities {
    app: OpenHarmonyApp,
    text: String,
    password: String,
    status: Arc<Mutex<String>>,
    child: bool,
    pen_status: String,
    file_dialog_pending: Arc<std::sync::atomic::AtomicBool>,
    events: std::sync::mpsc::Receiver<winit_ohos::events::PlatformEvent>,
}

impl Capabilities {
    pub fn new(app: OpenHarmonyApp) -> Self {
        Self {
            app,
            text: "Hello 鸿蒙 😀".into(),
            password: String::new(),
            status: Arc::new(Mutex::new("Ready".into())),
            child: false,
            pen_status: "Pen: waiting for input".into(),
            file_dialog_pending: Arc::default(),
            events: winit_ohos::events::subscribe(),
        }
    }

    fn run(
        &self,
        ctx: &egui::Context,
        future: impl std::future::Future<Output = String> + Send + 'static,
    ) {
        let status = Arc::clone(&self.status);
        let ctx = ctx.clone();
        task::spawn(async move {
            let result = future.await;
            log::info!("OHOS capability: {result}");
            *status.lock() = result;
            ctx.request_repaint();
        });
    }

    fn show_file_dialog(&self, ctx: &egui::Context, kind: &'static str) {
        let mut options = FileDialogOptions::new(kind);
        if kind == "save-file" {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            options = options.suggested_name(format!("egui-{timestamp}.txt"));
        }
        let future = self.app.show_file_dialog(options);
        let pending = Arc::clone(&self.file_dialog_pending);
        pending.store(true, std::sync::atomic::Ordering::Relaxed);
        let text = self.text.clone();
        self.run(ctx, async move {
            let result: Result<String, String> = async {
                let response = future.await.map_err(|error| error.to_string())?;
                let paths =
                    winit_ohos::file_access::persist_paths(&response.files, kind == "save-file")?;
                let Some(path) = paths.first() else {
                    return Ok("Cancelled".to_owned());
                };
                match kind {
                    "save-file" => {
                        std::fs::write(path, text.as_bytes()).map_err(|error| error.to_string())?;
                        Ok(format!("Saved {} bytes to {}", text.len(), path.display()))
                    }
                    "open-folder" => {
                        let entries = std::fs::read_dir(path)
                            .map_err(|error| error.to_string())?
                            .collect::<std::io::Result<Vec<_>>>()
                            .map_err(|error| error.to_string())?;
                        Ok(format!(
                            "Folder {}: {} entries",
                            path.display(),
                            entries.len()
                        ))
                    }
                    _ => {
                        use std::io::Read as _;
                        let mut text = String::new();
                        std::fs::File::open(path)
                            .map_err(|error| error.to_string())?
                            .take(4096)
                            .read_to_string(&mut text)
                            .map_err(|error| error.to_string())?;
                        Ok(format!("Read {}: {text}", path.display()))
                    }
                }
            }
            .await;
            pending.store(false, std::sync::atomic::Ordering::Relaxed);
            format!("Files: {result:?}")
        });
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Pen { phase, .. } = event {
                    self.pen_status = format!("{event:?}");
                    if *phase != egui::TouchPhase::Move {
                        log::info!("OHOS native pen: {}", self.pen_status);
                    }
                }
            }
        });
        while let Ok(event) = self.events.try_recv() {
            let message = format!("{event:?}");
            log::info!("OHOS platform event: {message}");
            *self.status.lock() = match event {
                winit_ohos::events::PlatformEvent::Menu { id } if id == "copy" => {
                    ctx.copy_text(self.text.clone());
                    "Copied from native menu".into()
                }
                winit_ohos::events::PlatformEvent::Want { uri, .. } => {
                    format!("Launch / deep link: {uri}")
                }
                _ => message,
            };
        }
        egui::Window::new("OHOS capabilities")
            .default_pos([8.0, 36.0])
            .default_width(310.0)
            .show(ctx, |ui| {
                ui.label("Native system integration");
                ui.label(&self.pen_status);
                ui.text_edit_singleline(&mut self.text);
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Password"),
                );
                ui.horizontal(|ui| {
                    if ui.button("Copy text").clicked() {
                        ctx.copy_text(self.text.clone());
                    }
                    if ui.button("Read clipboard").clicked() {
                        let client = ClipboardClient::new(&self.app);
                        let app = self.app.clone();
                        self.run(ctx, async move {
                            let _ = app
                                .request_permission("ohos.permission.READ_PASTEBOARD")
                                .await;
                            match client {
                                Ok(client) => format!("Clipboard: {:?}", client.read_text().await),
                                Err(error) => error.to_string(),
                            }
                        });
                    }
                    if ui.button("Copy image").clicked() {
                        ctx.copy_image(egui::ColorImage::filled([32, 32], egui::Color32::GREEN));
                    }
                });
                ui.horizontal(|ui| {
                    for (label, kind) in [
                        ("Open file", "open-file"),
                        ("Save file", "save-file"),
                        ("Folder", "open-folder"),
                    ] {
                        if ui.add_enabled(!self.file_dialog_pending.load(std::sync::atomic::Ordering::Relaxed), egui::Button::new(label)).clicked() {
                            self.show_file_dialog(ctx, kind);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Open website").clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab("https://www.egui.rs"));
                    }
                    if ui.button("System dialog").clicked() {
                        let client = WindowClient::new(&self.app);
                        self.run(ctx, async move {
                            match client {
                                Ok(client) => format!(
                                    "Dialog: {:?}",
                                    client
                                        .show_prompt(WindowPromptRequest {
                                            window_id: 0,
                                            message: "egui native dialog".into(),
                                            detail: Some("中文与 Unicode 😀".into()),
                                            level: 0,
                                            buttons: vec!["OK".into(), "Cancel".into()],
                                            cancel_index: Some(1),
                                            anchor: None
                                        })
                                        .await
                                ),
                                Err(error) => error.to_string(),
                            }
                        });
                    }
                });
                ui.checkbox(&mut self.child, "Native child viewport");
                ui.horizontal(|ui| {
                    if ui.button("Maximize").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                    }
                    if ui.button("Restore").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                    }
                    if ui.button("Minimize").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Fullscreen").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    }
                    if ui.button("Windowed").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    }
                });
                ui.collapsing("More system capabilities", |ui| {
                    use winit_ohos::plugins::{menu::{MenuClient, MenuPopupRequest}, notification::{NotificationClient, ShowNotificationRequest}, system_state::SystemStateClient, process::Process};
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Inspect clipboard").clicked() {
                            let client = ClipboardClient::new(&self.app);
                            self.run(ctx, async move { match client {
                                Ok(client) => match client.read_content().await {
                                    Ok(content) => format!("Clipboard: text={:?}; PNG={} bytes; URIs={:?}", content.text, content.png.as_ref().map_or(0, Vec::len), content.uris),
                                    Err(error) => error.to_string()
                                }, Err(error) => error.to_string()
                            }});
                        }
                        if ui.button("Native menu").clicked() {
                            let client = MenuClient::new(&self.app);
                            self.run(ctx, async move { match client {
                                Ok(client) => format!("Menu: {:?}", client.popup(MenuPopupRequest {
                                    json_data: r#"[{"id":"hello","type":"normal","text":"Hello 鸿蒙 😀","enabled":true},{"id":"copy","type":"normal","text":"Copy text","enabled":true}]"#.into(),
                                    x: None, y: None, window_id: "main".into()
                                }).await), Err(error) => error.to_string()
                            }});
                        }
                        if ui.button("Notify").clicked() {
                            let client = NotificationClient::new(&self.app);
                            self.run(ctx, async move { match client {
                                Ok(client) => format!("Notification: {:?}", client.show(ShowNotificationRequest {
                                    tag: "egui-test".into(), title: "egui 鸿蒙".into(), body: "Native notification 😀".into(), actions: vec![]
                                }).await), Err(error) => error.to_string()
                            }});
                        }
                        if ui.button("Dismiss notification").clicked() {
                            let client = NotificationClient::new(&self.app);
                            self.run(ctx, async move { match client {
                                Ok(client) => format!("Dismiss: {:?}", client.dismiss("egui-test".into()).await), Err(error) => error.to_string()
                            }});
                        }
                        if ui.button("System state").clicked() {
                            let client = SystemStateClient::new(&self.app);
                            self.run(ctx, async move { match client {
                                Ok(client) => format!("Displays: {:?}; thermal: {:?}; area: {:?}", client.displays().await, client.thermal_level().await, client.available_area().await), Err(error) => error.to_string()
                            }});
                        }
                        if ui.button("Restart app").clicked() {
                            let client = Process::new(&self.app);
                            self.run(ctx, async move { match client {
                                Ok(client) => format!("Restart: {:?}", client.restart().await), Err(error) => error.to_string()
                            }});
                        }
                        if ui.button("Close app").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.add(egui::Button::new("Drag sample file").sense(egui::Sense::drag())).drag_started() {
                            let result = self.app.pref_path().ok_or_else(|| "App files directory unavailable".to_owned()).and_then(|directory| {
                                let path = std::path::Path::new(&directory).join("egui-drag.txt");
                                std::fs::write(&path, "egui native drag: 鸿蒙 😀\n").map_err(|error| error.to_string())?;
                                let uri = winit_ohos::drag_drop::file_uri(&path).ok_or_else(|| "Cannot encode file URI".to_owned())?;
                                winit_ohos::drag_drop::start_file_drag_with_options(&self.app, winit::window::WindowId::from_raw(0), vec![uri],
                                    vec![winit::event_loop::DndAction::Copy, winit::event_loop::DndAction::Move],
                                    Some(openharmony_ability::DragPreview { rgba: [255, 80, 20, 220].repeat(48 * 32), width: 48, height: 32, hotspot_x: 12.0, hotspot_y: 8.0 }))
                                    .map(|id| format!("Native file drag: {id:?}")).map_err(|error| error.to_string())
                            });
                            log::info!("OHOS capability: {result:?}");
                            *self.status.lock() = format!("{result:?}");
                        }
                        if ui.button("Window icon").clicked() {
                            let client = WindowClient::new(&self.app);
                            self.run(ctx, async move {
                                match client {
                                    Ok(client) => format!("Window icon: {:?}", client.set_window_icon(winit_ohos::plugins::window::WindowIconRequest { window_id: 0, size: 32, rgba: [250, 130, 30, 255].repeat(32 * 32) }).await),
                                    Err(error) => error.to_string(),
                                }
                            });
                        }
                        if ui.button("Custom cursor").clicked() {
                            let client = WindowClient::new(&self.app);
                            self.run(ctx, async move {
                                match client {
                                    Ok(client) => format!("Custom cursor: {:?}", client.set_custom_cursor(
                                        winit_ohos::plugins::window::CustomCursorRequest {
                                            window_id: 0, rgba: [255, 80, 20, 255].repeat(24 * 24),
                                            width: 24, height: 24, hotspot_x: 12, hotspot_y: 12,
                                        }).await),
                                    Err(error) => error.to_string(),
                                }
                            });
                        }
                        if ui.button("Screenshot").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                        }
                    });
                });
                egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    ui.label(self.status.lock().as_str());
                });
                let dropped_files = ctx.input(|input| {
                    for event in &input.events {
                        if let egui::Event::Pen { .. } = event {
                            log::info!("OHOS native pen: {event:?}");
                        }
                        if let egui::Event::AccessKitActionRequest(request) = event {
                            log::info!("OHOS accessibility action: {request:?}");
                        }
                        if let egui::Event::Screenshot { image, .. } = event {
                            let message = format!("Screenshot: {} × {}", image.width(), image.height());
                            log::info!("OHOS capability: {message}");
                            *self.status.lock() = message;
                        }
                    }
                    input.raw.dropped_files.clone()
                });
                // UI layout reads the context too; release the input lock first.
                for file in dropped_files {
                    ui.label(format!("Dropped: {:?}", file.path));
                    log::info!("OHOS file drop: {:?}", file.path);
                }
            });
        if self.child {
            let open = Arc::new(std::sync::atomic::AtomicBool::new(true));
            let child_open = Arc::clone(&open);
            // Leave room for the native titlebar and initial window offset on phones.
            // Monitor size stays stable when the soft keyboard resizes the root.
            let child_size = ctx.input(|input| {
                let preferred = egui::vec2(360.0, 240.0);
                input.viewport().monitor_size.map_or(preferred, |screen| {
                    preferred.min((screen - egui::vec2(64.0, 96.0)).max(egui::vec2(160.0, 120.0)))
                })
            });
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("ohos-child"),
                egui::ViewportBuilder::default()
                    .with_title("egui native child")
                    .with_inner_size(child_size),
                move |ui, _| {
                    let ctx = ui.ctx().clone();
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.heading("Native child viewport");
                        ui.label("Independent rendering, input and lifecycle");
                        let mut text = ctx.data_mut(|d| {
                            d.get_temp::<String>(egui::Id::new("child-text"))
                                .unwrap_or_else(|| "子窗口 😀".into())
                        });
                        ui.text_edit_singleline(&mut text);
                        ctx.data_mut(|d| d.insert_temp(egui::Id::new("child-text"), text));
                        let dropped_files = ctx.input(|input| input.raw.dropped_files.clone());
                        for file in dropped_files {
                            ui.label(format!("Dropped: {:?}", file.path));
                            log::info!("OHOS child file drop: {:?}", file.path);
                        }
                        if ui.button("Close child").clicked()
                            || ctx.input(|i| i.viewport().close_requested())
                        {
                            child_open.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    });
                },
            );
            self.child = open.load(std::sync::atomic::Ordering::Relaxed);
        }
    }
}

#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![cfg_attr(
    target_env = "ohos",
    expect(clippy::unwrap_used, reason = "application startup must succeed")
)]

use eframe::{CreationContext, egui};
#[cfg(target_env = "ohos")]
mod capabilities;

#[cfg(target_env = "ohos")]
use openharmony_ability_derive::ability;

#[cfg(target_env = "ohos")]
#[ability]
fn openharmony(app: winit::platform::ohos::ability::OpenHarmonyApp) {
    ohos_hilog_binding::log::init_once(
        ohos_hilog_binding::log::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let options = eframe::NativeOptions {
        openharmony_app: Some(app.clone()),
        renderer: {
            #[cfg(feature = "wgpu")]
            {
                eframe::Renderer::Wgpu
            }
            #[cfg(not(feature = "wgpu"))]
            {
                eframe::Renderer::Glow
            }
        },
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(move |cc| {
            let mut demo = MyApp::new(cc);
            demo.capabilities = Some(capabilities::Capabilities::new(app));
            Ok(Box::new(demo))
        }),
    )
    .unwrap();
}

pub struct MyApp {
    demo: egui_demo_lib::DemoWindows,
    #[cfg(target_env = "ohos")]
    capabilities: Option<capabilities::Capabilities>,
}

impl MyApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            demo: egui_demo_lib::DemoWindows::default(),
            #[cfg(target_env = "ohos")]
            capabilities: None,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.demo.ui(ui);
            #[cfg(target_env = "ohos")]
            if let Some(capabilities) = &mut self.capabilities {
                capabilities.ui(ui.ctx());
            }
        });
    }
}

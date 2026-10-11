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
        #[cfg(target_env = "ohos")]
        install_system_font(&cc.egui_ctx);
        Self {
            demo: egui_demo_lib::DemoWindows::default(),
            #[cfg(target_env = "ohos")]
            capabilities: None,
        }
    }
}

#[cfg(target_env = "ohos")]
fn install_system_font(ctx: &egui::Context) {
    // The bundled egui fonts do not include CJK. Use the device font without
    // redistributing it, retaining the bundled fonts for Latin text and icons.
    for path in [
        "/system/fonts/HarmonyOS_Sans_SC.ttf",
        "/system/fonts/NotoSansCJK-Regular.ttc",
    ] {
        if let Ok(data) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            let name = "ohos-system-cjk".to_owned();
            fonts.font_data.insert(
                name.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );
            for family in fonts.families.values_mut() {
                family.push(name.clone());
            }
            ctx.set_fonts(fonts);
            break;
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.demo.ui(ui);
            #[cfg(target_env = "ohos")]
            if let Some(capabilities) = &mut self.capabilities {
                capabilities.ui(ui.ctx());
            }
        });
    }
}

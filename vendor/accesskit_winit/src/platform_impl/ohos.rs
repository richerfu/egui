// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in LICENSE-APACHE).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use ohos_accessibility_binding::Provider;
use winit::{
    event::WindowEvent, event_loop::ActiveEventLoop, platform::ohos::WindowExtOpenHarmony,
    window::Window,
};

pub struct Adapter {
    native: Option<accesskit_ohos::Adapter<'static>>,
    deactivation: Box<dyn DeactivationHandler + Send>,
}

impl Adapter {
    pub fn new(
        _event_loop: &dyn ActiveEventLoop,
        window: &dyn Window,
        activation: impl ActivationHandler + Send + 'static,
        action: impl ActionHandler + Send + 'static,
        deactivation: impl DeactivationHandler + Send + 'static,
    ) -> Self {
        let raw = window.openharmony_app().with_xcomponent_for(
            window.id().into_raw() as i64,
            |component| {
                component
                    .native_xcomponent()
                    .accessibility_provider()
                    .map(|provider| provider.as_raw() as usize)
            },
        );
        let native = raw.and_then(|raw| {
            let result = raw.and_then(|raw| {
                // The XComponent owns this provider. Eframe drops its adapter
                // during destroy_surfaces, before Ability destroys that component.
                let provider = unsafe { Provider::from_raw(raw as *mut _) }?;
                accesskit_ohos::Adapter::new_with_instance(
                    provider,
                    &format!("egui-{}", {
                        static NEXT: std::sync::atomic::AtomicU64 =
                            std::sync::atomic::AtomicU64::new(1);
                        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    }),
                    activation,
                    action,
                )
            });
            match result {
                Ok(adapter) => {
                    log::info!("OHOS accessibility registered window={:?}", window.id());
                    Some(adapter)
                }
                Err(error) => {
                    log::warn!("OHOS accessibility initialization: {error}");
                    None
                }
            }
        });
        let mut adapter = Self {
            native,
            deactivation: Box::new(deactivation),
        };
        adapter.process_event(window, &WindowEvent::Focused(window.has_focus()));
        adapter
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(native) = &self.native {
            if let Err(error) = native.update_if_active(updater) {
                log::warn!("OHOS accessibility update: {error}");
            }
        }
    }

    pub fn process_event(&mut self, _window: &dyn Window, event: &WindowEvent) {
        if let Some(native) = &self.native {
            if let WindowEvent::Focused(focused) = event {
                let _ = native.set_host_focus_state(*focused);
            }
            // ArkUI adds the XComponent's transformed position and its window
            // offset to provider rectangles. AccessKit already supplies physical
            // surface coordinates; adding those offsets here applies them twice.
            let _ = native.set_screen_offset(0, 0);
        }
        if matches!(event, WindowEvent::Destroyed) {
            self.release();
        }
    }

    fn release(&mut self) {
        if self.native.take().is_some() {
            self.deactivation.deactivate_accessibility();
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        self.release();
    }
}

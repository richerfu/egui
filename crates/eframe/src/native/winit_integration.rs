use std::{sync::Arc, time::Instant};

use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use egui::ViewportId;
#[cfg(feature = "accesskit")]
use egui_winit::accesskit_winit;

/// Create an egui context, restoring it from storage if possible.
pub fn create_egui_context(storage: Option<&dyn crate::Storage>) -> egui::Context {
    profiling::function_scope!();

    pub const IS_DESKTOP: bool = cfg!(any(
        target_os = "freebsd",
        all(target_os = "linux", not(target_env = "ohos")),
        target_os = "macos",
        target_os = "openbsd",
        target_os = "windows",
    ));

    let egui_ctx = egui::Context::default();

    egui_ctx.set_embed_viewports(!(IS_DESKTOP || cfg!(target_env = "ohos")));

    egui_ctx.options_mut(|o| {
        // eframe supports multi-pass (Context::request_discard).
        #[expect(clippy::unwrap_used)]
        {
            o.max_passes = 2.try_into().unwrap();
        }
    });

    let memory = crate::native::epi_integration::load_egui_memory(storage).unwrap_or_default();
    egui_ctx.memory_mut(|mem| *mem = memory);

    egui_ctx
}

/// The custom even `eframe` uses with the [`winit`] event loop.
#[derive(Debug)]
pub enum UserEvent {
    /// A repaint is requested.
    RequestRepaint {
        /// What to repaint.
        viewport_id: ViewportId,

        /// When to repaint.
        when: Instant,

        /// What the cumulative pass number was when the repaint was _requested_.
        cumulative_pass_nr: u64,
    },

    /// A request related to [`accesskit`](https://accesskit.dev/).
    #[cfg(feature = "accesskit")]
    AccessKitActionRequest(accesskit_winit::Event),
}

#[cfg(feature = "accesskit")]
impl From<accesskit_winit::Event> for UserEvent {
    fn from(inner: accesskit_winit::Event) -> Self {
        Self::AccessKitActionRequest(inner)
    }
}

/// Typed event channel paired with winit's coalescing wake-up proxy.
#[derive(Clone)]
pub struct UserEventProxy {
    sender: std::sync::mpsc::Sender<UserEvent>,
    proxy: winit::event_loop::EventLoopProxy,
}

impl UserEventProxy {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop,
    ) -> (Self, std::sync::mpsc::Receiver<UserEvent>) {
        let (sender, receiver) = std::sync::mpsc::channel();
        (
            Self {
                sender,
                proxy: event_loop.create_proxy(),
            },
            receiver,
        )
    }

    pub fn send_event(
        &self,
        event: UserEvent,
    ) -> Result<(), std::sync::mpsc::SendError<UserEvent>> {
        self.sender.send(event)?;
        self.proxy.wake_up();
        Ok(())
    }
}

pub trait WinitApp {
    fn take_event_receiver(&mut self) -> std::sync::mpsc::Receiver<UserEvent>;

    fn egui_ctx(&self) -> Option<&egui::Context>;

    fn window(&self, window_id: WindowId) -> Option<Arc<dyn Window>>;

    fn window_id_from_viewport_id(&self, id: ViewportId) -> Option<WindowId>;

    fn save(&mut self);

    fn save_and_destroy(&mut self);

    fn run_ui_and_paint(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
    ) -> crate::Result<EventResult>;

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) -> crate::Result<EventResult>;

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) -> crate::Result<EventResult>;

    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        device_id: Option<winit::event::DeviceId>,
        event: winit::event::DeviceEvent,
    ) -> crate::Result<EventResult>;

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) -> crate::Result<EventResult>;

    #[cfg(feature = "accesskit")]
    fn on_accesskit_event(&mut self, event: accesskit_winit::Event) -> crate::Result<EventResult>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventResult {
    Wait,

    /// Causes a synchronous repaint inside the event handler. This should only
    /// be used in special situations if the window must be repainted while
    /// handling a specific event. This occurs on Windows when handling resizes.
    ///
    /// `RepaintNow` creates a new frame synchronously, and should therefore
    /// only be used for extremely urgent repaints.
    RepaintNow(WindowId),

    /// Queues a repaint for once the event loop handles its next redraw. Exists
    /// so that multiple input events can be handled in one frame. Does not
    /// cause any delay like `RepaintNow`.
    RepaintNext(WindowId),

    RepaintAt(WindowId, Instant),

    /// Causes a save of the client state when the persistence feature is enabled.
    Save,

    /// Starts the process of ending eframe execution whilst allowing for proper
    /// clean up of resources.
    ///
    /// # Warning
    /// This event **must** occur before [`Exit`] to correctly exit eframe code.
    /// If in doubt, return this event.
    ///
    /// [`Exit`]: [EventResult::Exit]
    CloseRequested,

    /// The event loop will exit, now.
    /// The correct circumstance to return this event is in response to a winit "Destroyed" event.
    ///
    /// # Warning
    /// The [`CloseRequested`] **must** occur before this event to ensure that winit
    /// is able to remove any open windows. Otherwise the window(s) will remain open
    /// until the program terminates.
    ///
    /// [`CloseRequested`]: EventResult::CloseRequested
    Exit,
}

#[cfg(feature = "accesskit")]
pub(crate) fn on_accesskit_window_event(
    egui_winit: &mut egui_winit::State,
    window_id: WindowId,
    event: &accesskit_winit::WindowEvent,
) -> EventResult {
    match event {
        accesskit_winit::WindowEvent::InitialTreeRequested => {
            egui_winit.egui_ctx().enable_accesskit();
            // Because we can't provide the initial tree synchronously
            // (because that would require the activation handler to access
            // the same mutable state as the winit event handler), some
            // AccessKit platform adapters will use a placeholder tree
            // until we send the first tree update. To minimize the possible
            // bad effects of that workaround, repaint and send the tree
            // immediately.
            EventResult::RepaintNow(window_id)
        }
        accesskit_winit::WindowEvent::ActionRequested(request) => {
            egui_winit.on_accesskit_action_request(request.clone());
            // As a form of user input, accessibility actions should cause
            // a repaint, but not until the next regular frame.
            EventResult::RepaintNext(window_id)
        }
        accesskit_winit::WindowEvent::AccessibilityDeactivated => {
            egui_winit.egui_ctx().disable_accesskit();
            // Disabling AccessKit support should have no visible effect,
            // so there's no need to repaint.
            EventResult::Wait
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug, Default)]
    struct CoalescingWake(AtomicBool);

    impl winit::event_loop::EventLoopProxyProvider for CoalescingWake {
        fn wake_up(&self) {
            self.0.store(true, Ordering::Release);
        }
    }

    #[test]
    fn coalesced_wake_preserves_all_cross_thread_repaint_requests() {
        let wake = Arc::new(CoalescingWake::default());
        let (sender, receiver) = std::sync::mpsc::channel();
        let proxy = UserEventProxy {
            sender,
            proxy: winit::event_loop::EventLoopProxy::new(
                Arc::clone(&wake) as Arc<dyn winit::event_loop::EventLoopProxyProvider>
            ),
        };
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let proxy = proxy.clone();
                scope.spawn(move || {
                    for pass in 0..25 {
                        assert!(
                            proxy
                                .send_event(UserEvent::RequestRepaint {
                                    viewport_id: ViewportId::ROOT,
                                    when: Instant::now(),
                                    cumulative_pass_nr: pass,
                                })
                                .is_ok()
                        );
                    }
                });
            }
        });
        assert!(wake.0.swap(false, Ordering::AcqRel));
        assert_eq!(receiver.try_iter().count(), 100);
        drop(receiver);
        assert!(
            proxy
                .send_event(UserEvent::RequestRepaint {
                    viewport_id: ViewportId::ROOT,
                    when: Instant::now(),
                    cumulative_pass_nr: 0,
                })
                .is_err()
        );
        assert!(!wake.0.load(Ordering::Acquire));
    }
}

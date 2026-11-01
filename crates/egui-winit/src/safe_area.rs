#[cfg(target_os = "ios")]
pub use ios::get_safe_area_insets;
#[cfg(target_env = "ohos")]
pub use ohos::get_safe_area_insets;

#[cfg(target_env = "ohos")]
mod ohos {
    use egui::{SafeAreaInsets, epaint::MarginF32};
    use winit::platform::ohos::{
        WindowExtOpenHarmony as _,
        ability::{AvoidAreaType, Rect},
    };

    fn edge_insets(
        surface_start: i32,
        surface_len: i32,
        avoid_start: i32,
        avoid_len: i32,
    ) -> (u32, u32) {
        if surface_len <= 0 || avoid_len <= 0 {
            return (0, 0);
        }
        let start = i64::from(surface_start);
        let len = i64::from(surface_len);
        let end = start + len;
        let avoid_start = i64::from(avoid_start);
        let avoid_end = avoid_start + i64::from(avoid_len);
        let near = if avoid_start <= start {
            (avoid_end - start).clamp(0, len)
        } else {
            0
        };
        let far = if avoid_end >= end {
            (end - avoid_start).clamp(0, len)
        } else {
            0
        };
        (near as u32, far as u32)
    }

    pub fn get_safe_area_insets(
        window: &winit::window::Window,
        pixels_per_point: f32,
    ) -> SafeAreaInsets {
        if u64::from(window.id()) != 0 {
            return SafeAreaInsets::default();
        }

        let app = window.openharmony_app();
        let content = window.content_rect();
        let vertical = |rect: Rect| {
            if rect.width <= 0
                || i64::from(rect.left) >= i64::from(content.left) + i64::from(content.width)
                || i64::from(rect.left) + i64::from(rect.width) <= i64::from(content.left)
            {
                return (0, 0);
            }
            edge_insets(content.top, content.height, rect.top, rect.height)
        };
        let horizontal = |rect: Rect| {
            if rect.height <= 0
                || i64::from(rect.top) >= i64::from(content.top) + i64::from(content.height)
                || i64::from(rect.top) + i64::from(rect.height) <= i64::from(content.top)
            {
                return (0, 0);
            }
            edge_insets(content.left, content.width, rect.left, rect.width)
        };

        let mut top = 0;
        let mut left = 0;
        let mut bottom = 0;
        let mut right = 0;

        #[expect(clippy::iter_over_hash_type)]
        for (area_type, area) in app.avoid_areas() {
            if !area.visible || matches!(area_type, AvoidAreaType::Keyboard) {
                continue;
            }
            top = top.max(vertical(area.top_rect).0);
            left = left.max(horizontal(area.left_rect).0);
            bottom = bottom.max(vertical(area.bottom_rect).1);
            right = right.max(horizontal(area.right_rect).1);
        }

        let ppp = if pixels_per_point > 0.0 && pixels_per_point.is_finite() {
            pixels_per_point
        } else {
            1.0
        };
        SafeAreaInsets(MarginF32 {
            top: top as f32 / ppp,
            left: left as f32 / ppp,
            right: right as f32 / ppp,
            bottom: bottom as f32 / ppp,
        })
    }
}

#[cfg(target_os = "ios")]
mod ios {
    use egui::{SafeAreaInsets, epaint::MarginF32};
    use objc2::{ClassType, rc::Retained};
    use objc2_foundation::{MainThreadMarker, NSObjectProtocol};
    use objc2_ui_kit::{UIApplication, UISceneActivationState, UIWindowScene};

    /// Gets the ios safe area insets.
    ///
    /// A safe area defines the area within a view that isn’t covered by a navigation bar, tab bar,
    /// toolbar, or other views a window might provide. Safe areas are essential for avoiding a
    /// device’s interactive and display features, like Dynamic Island on iPhone or the camera
    /// housing on some Mac models.
    ///
    /// Once winit v0.31 has been released this can be removed in favor of
    /// `winit::Window::safe_area`.
    pub fn get_safe_area_insets() -> SafeAreaInsets {
        let Some(main_thread_marker) = MainThreadMarker::new() else {
            log::error!("Getting safe area insets needs to be performed on the main thread");
            return SafeAreaInsets::default();
        };

        let app = UIApplication::sharedApplication(main_thread_marker);

        #[expect(unsafe_code)]
        unsafe {
            // Look for the first window scene that's in the foreground
            for scene in app.connectedScenes() {
                if scene.isKindOfClass(UIWindowScene::class())
                    && matches!(
                        scene.activationState(),
                        UISceneActivationState::ForegroundActive
                            | UISceneActivationState::ForegroundInactive
                    )
                {
                    // SAFETY: class kind was checked above with `isKindOfClass`
                    let window_scene = Retained::cast_unchecked::<UIWindowScene>(scene.clone());
                    if let Some(window) = window_scene.keyWindow() {
                        let insets = window.safeAreaInsets();
                        return SafeAreaInsets(MarginF32 {
                            top: insets.top as f32,
                            left: insets.left as f32,
                            right: insets.right as f32,
                            bottom: insets.bottom as f32,
                        });
                    }
                }
            }
        }

        SafeAreaInsets::default()
    }
}

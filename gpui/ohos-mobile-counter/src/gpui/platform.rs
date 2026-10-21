//! Application-level GPUI platform backed by the embedded ArkUI host.

use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::gpui_core::*;
use futures::channel::oneshot;
use uuid::Uuid;

use super::dispatcher::Dispatcher;
use super::window::OhosWindow;
use crate::host::surface;
use crate::sdk_ffi::display;

#[derive(Debug)]
pub(crate) struct OhosDisplay {
    id: DisplayId,
    bounds: Bounds<Pixels>,
    refresh_interval: Option<Duration>,
}

impl PlatformDisplay for OhosDisplay {
    fn id(&self) -> DisplayId {
        self.id
    }
    fn uuid(&self) -> Result<Uuid> {
        Err(crate::gpui_core::private::anyhow::anyhow!(
            "stable OHOS display UUID unavailable"
        ))
    }
    fn bounds(&self) -> Bounds<Pixels> {
        self.bounds
    }
    fn refresh_interval(&self) -> Option<Duration> {
        self.refresh_interval
    }
}

pub(crate) struct OhosPlatform {
    dispatcher: Arc<Dispatcher>,
    display_changed: RefCell<Option<Box<dyn FnMut()>>>,
    clipboard: RefCell<Option<ClipboardItem>>,
}

impl OhosPlatform {
    pub(crate) fn new(dispatcher: Arc<Dispatcher>) -> Self {
        Self {
            dispatcher,
            display_changed: RefCell::new(None),
            clipboard: RefCell::new(None),
        }
    }

    fn display(&self) -> Rc<dyn PlatformDisplay> {
        let info = display::default_display();
        let fallback = surface::snapshot()
            .map(|s| s.logical_dimensions())
            .unwrap_or((1.0, 1.0));
        let (id, width, height, refresh_interval) = match info {
            Some(info) => (
                DisplayId::new(info.id),
                info.width_pixels as f32 / info.density,
                info.height_pixels as f32 / info.density,
                info.refresh_hz
                    .map(|rate| Duration::from_secs_f64(1.0 / f64::from(rate))),
            ),
            None => (DisplayId::new(1), fallback.0, fallback.1, None),
        };
        Rc::new(OhosDisplay {
            id,
            bounds: Bounds::new(point(px(0.0), px(0.0)), size(px(width), px(height))),
            refresh_interval,
        })
    }

    pub(crate) fn surface_changed(&self) {
        if let Some(callback) = self.display_changed.borrow_mut().as_mut() {
            callback();
        }
    }
}

struct OhosKeyboardLayout;
impl PlatformKeyboardLayout for OhosKeyboardLayout {
    fn id(&self) -> &str {
        "ohos-default"
    }
    fn name(&self) -> &str {
        "OHOS keyboard"
    }
}

fn unsupported<T>(message: &'static str) -> oneshot::Receiver<Result<T>> {
    let (tx, rx) = oneshot::channel();
    let _ = tx.send(Err(crate::gpui_core::private::anyhow::anyhow!(message)));
    rx
}

impl Platform for OhosPlatform {
    fn background_executor(&self) -> BackgroundExecutor {
        BackgroundExecutor::new(self.dispatcher.clone())
    }
    fn foreground_executor(&self) -> ForegroundExecutor {
        ForegroundExecutor::new(self.dispatcher.clone())
    }
    fn text_system(&self) -> Arc<dyn PlatformTextSystem> {
        Arc::new(NoopTextSystem::new())
    }
    fn run(&self, on_finish_launching: Box<dyn FnOnce()>) {
        on_finish_launching();
    }
    fn quit(&self) {}
    fn restart(&self, _binary_path: Option<PathBuf>, _arguments: Vec<OsString>) {}
    fn activate(&self, _ignoring_other_apps: bool) {}
    fn hide(&self) {}
    fn hide_other_apps(&self) {}
    fn unhide_other_apps(&self) {}
    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> {
        vec![self.display()]
    }
    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(self.display())
    }
    fn on_displays_changed(&self, callback: Box<dyn FnMut()>) {
        *self.display_changed.borrow_mut() = Some(callback);
    }
    fn active_window(&self) -> Option<AnyWindowHandle> {
        None
    }
    fn open_window(
        &self,
        _handle: AnyWindowHandle,
        _options: WindowParams,
    ) -> Result<Box<dyn PlatformWindow>> {
        let snapshot = surface::snapshot().ok_or_else(|| {
            crate::gpui_core::private::anyhow::anyhow!("XComponent surface not ready")
        })?;
        Ok(Box::new(OhosWindow::new(snapshot, self.display())))
    }
    fn window_appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }
    fn open_url(&self, _url: &str) {}
    fn on_open_urls(&self, _callback: Box<dyn FnMut(Vec<String>)>) {}
    fn register_url_scheme(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Err(crate::gpui_core::private::anyhow::anyhow!(
            "URL schemes unsupported"
        )))
    }
    fn prompt_for_paths(
        &self,
        _options: PathPromptOptions,
    ) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>> {
        unsupported("file picker unsupported")
    }
    fn prompt_for_new_path(
        &self,
        _directory: &Path,
        _suggested_name: Option<&str>,
    ) -> oneshot::Receiver<Result<Option<PathBuf>>> {
        unsupported("file picker unsupported")
    }
    fn can_select_mixed_files_and_dirs(&self) -> bool {
        false
    }
    fn reveal_path(&self, _path: &Path) {}
    fn open_with_system(&self, _path: &Path) {}
    fn on_quit(&self, _callback: Box<dyn FnMut() -> bool>) {}
    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}
    fn on_system_sleep(&self, _callback: Box<dyn FnMut()>) {}
    fn on_system_wake(&self, _callback: Box<dyn FnMut()>) {}
    fn set_menus(&self, _menus: Vec<Menu>, _keymap: &Keymap) {}
    fn set_dock_menu(&self, _menu: Vec<MenuItem>, _keymap: &Keymap) {}
    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn Action)>) {}
    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}
    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn Action) -> bool>) {}
    fn thermal_state(&self) -> ThermalState {
        ThermalState::Nominal
    }
    fn on_thermal_state_change(&self, _callback: Box<dyn FnMut()>) {}
    fn prevent_idle_sleep(&self, _reason: &str) -> Task<Result<ActivityGuard>> {
        Task::ready(Ok(ActivityGuard::noop()))
    }
    fn app_path(&self) -> Result<PathBuf> {
        Err(crate::gpui_core::private::anyhow::anyhow!(
            "application path unavailable"
        ))
    }
    fn path_for_auxiliary_executable(&self, _name: &str) -> Result<PathBuf> {
        Err(crate::gpui_core::private::anyhow::anyhow!(
            "auxiliary executable unavailable"
        ))
    }
    fn set_cursor_style(&self, _style: CursorStyle) {}
    fn hide_cursor_until_mouse_moves(&self) {}
    fn is_cursor_visible(&self) -> bool {
        false
    }
    fn should_auto_hide_scrollbars(&self) -> bool {
        true
    }
    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        self.clipboard.borrow().clone()
    }
    fn write_to_clipboard(&self, item: ClipboardItem) {
        *self.clipboard.borrow_mut() = Some(item);
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn read_from_primary(&self) -> Option<ClipboardItem> {
        None
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn write_to_primary(&self, _item: ClipboardItem) {}
    fn write_credentials(&self, _url: &str, _username: &str, _password: &[u8]) -> Task<Result<()>> {
        Task::ready(Err(crate::gpui_core::private::anyhow::anyhow!(
            "credential store unsupported"
        )))
    }
    fn read_credentials(&self, _url: &str) -> Task<Result<Option<(String, Vec<u8>)>>> {
        Task::ready(Err(crate::gpui_core::private::anyhow::anyhow!(
            "credential store unsupported"
        )))
    }
    fn delete_credentials(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Err(crate::gpui_core::private::anyhow::anyhow!(
            "credential store unsupported"
        )))
    }
    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> {
        Box::new(OhosKeyboardLayout)
    }
    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> {
        Rc::new(DummyKeyboardMapper)
    }
    fn on_keyboard_layout_change(&self, _callback: Box<dyn FnMut()>) {}
}

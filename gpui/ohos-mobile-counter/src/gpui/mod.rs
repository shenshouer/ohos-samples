//! GPUI's embedded OHOS platform. The host owns the UI loop and surface.

mod dispatcher;
mod platform;
mod window;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, ThreadId};

use crate::gpui_core::{
    AnyWindowHandle, AppContext, Application, ApplicationHandle, Context, IntoElement, Platform,
    Render, Window, WindowOptions, div, px, size,
};

use crate::host::logging::log;
use crate::sdk_ffi::{display, hilog};

use self::dispatcher::Dispatcher;
use self::platform::OhosPlatform;

struct Runtime {
    app: ApplicationHandle,
    dispatcher: Arc<Dispatcher>,
    platform: Rc<OhosPlatform>,
    window: Option<AnyWindowHandle>,
    window_generation: Option<u64>,
}

struct BootstrapRoot;
impl Render for BootstrapRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SurfaceEvent {
    Created(u64),
    Changed(u64),
    Destroyed(u64),
}

static HOST_EVENTS: Mutex<VecDeque<SurfaceEvent>> = Mutex::new(VecDeque::new());
static UI_THREAD: OnceLock<ThreadId> = OnceLock::new();

fn on_ui_thread() -> bool {
    UI_THREAD
        .get()
        .is_some_and(|id| *id == thread::current().id())
}

pub(crate) fn enqueue_surface(event: SurfaceEvent) {
    if let Ok(mut queue) = HOST_EVENTS.lock() {
        queue.push_back(event);
    }
}

thread_local! {
    static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
}

/// Called from Node-API on the ArkUI thread, before XComponent events are drained.
pub(crate) fn start() -> u32 {
    let expected = UI_THREAD.get_or_init(|| thread::current().id());
    if *expected != thread::current().id() {
        log(
            hilog::LOG_ERROR,
            "GPUI start rejected off ArkUI thread".into(),
        );
        return 0;
    }
    RUNTIME.with(|slot| {
        if slot.borrow().is_some() {
            return 1;
        }
        let dispatcher = Dispatcher::new();
        log(
            hilog::LOG_INFO,
            format!("GPUI display SDK info: {:?}", display::default_display()),
        );
        let platform = Rc::new(OhosPlatform::new(dispatcher.clone()));
        let app =
            Application::with_platform(platform.clone() as Rc<dyn Platform>).run_embedded(|cx| {
                log(hilog::LOG_INFO, "GPUI embedded launch completed".into());
                cx.foreground_executor()
                    .spawn(async {
                        log(
                            hilog::LOG_INFO,
                            "GPUI foreground task ran on ArkUI pump".into(),
                        );
                    })
                    .detach();
            });
        *slot.borrow_mut() = Some(Runtime {
            app,
            dispatcher,
            platform,
            window: None,
            window_generation: None,
        });
        #[cfg(debug_assertions)]
        {
            let _ = thread::Builder::new()
                .name("gpui-ohos-thread-check".into())
                .spawn(|| {
                    let result = pump();
                    log(
                        hilog::LOG_INFO,
                        format!("GPUI diagnostic off-thread pump result={result}"),
                    );
                });
        }
        1
    })
}

pub(crate) fn pump() -> u32 {
    if !on_ui_thread() {
        log(
            hilog::LOG_WARN,
            "GPUI pump rejected off ArkUI thread".into(),
        );
        return 0;
    }
    let Some(dispatcher) = RUNTIME.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|runtime| runtime.dispatcher.clone())
    }) else {
        return 0;
    };
    if !dispatcher.on_main_thread() {
        log(
            hilog::LOG_ERROR,
            "GPUI pump rejected off ArkUI thread".into(),
        );
        return 0;
    }
    let mut events = 0;
    while events < 64 {
        let event = HOST_EVENTS
            .lock()
            .ok()
            .and_then(|mut queue| queue.pop_front());
        let Some(event) = event else { break };
        let (kind, generation) = match event {
            SurfaceEvent::Created(n) => ("created", n),
            SurfaceEvent::Changed(n) => ("changed", n),
            SurfaceEvent::Destroyed(n) => ("destroyed", n),
        };
        log(
            hilog::LOG_INFO,
            format!(
                "GPUI host event {kind} generation={generation} pump_thread={:?}",
                std::thread::current().id()
            ),
        );
        RUNTIME.with(|slot| {
            let mut borrowed = slot.borrow_mut();
            let Some(runtime) = borrowed.as_mut() else {
                return;
            };
            runtime.platform.surface_changed();
            match event {
                SurfaceEvent::Created(_) | SurfaceEvent::Changed(_) => {
                    if let Some(snapshot) = crate::host::surface::snapshot() {
                        if snapshot.generation != generation {
                            return;
                        }
                        if runtime
                            .window_generation
                            .is_some_and(|old| old != generation)
                        {
                            if let Some(handle) = runtime.window.take() {
                                let _ = runtime.app.update(|cx| {
                                    handle.update(cx, |_, window, _| window.remove_window())
                                });
                            }
                            runtime.window_generation = None;
                        }
                        if let Some(handle) = runtime.window {
                            let resized = runtime.app.update(|cx| {
                                handle.update(cx, |_, window, _| {
                                    let (width, height) = snapshot.logical_dimensions();
                                    window.resize(size(px(width), px(height)));
                                })
                            });
                            if let Err(error) = resized {
                                log(
                                    hilog::LOG_WARN,
                                    format!("GPUI window resize failed: {error}"),
                                );
                                runtime.window = None;
                                runtime.window_generation = None;
                            }
                        }
                        if runtime.window.is_none() {
                            match runtime.app.update(|cx| {
                                cx.open_window(WindowOptions::default(), |_, cx| {
                                    cx.new(|_| BootstrapRoot)
                                })
                            }) {
                                Ok(handle) => {
                                    runtime.window = Some(handle.into());
                                    runtime.window_generation = Some(generation);
                                    log(
                                        hilog::LOG_INFO,
                                        format!(
                                            "GPUI window opened generation={} logical_size={:?} buffer_pixels={}x{}",
                                            snapshot.generation,
                                            snapshot.logical_dimensions(),
                                            snapshot.pixel_width,
                                            snapshot.pixel_height
                                        ),
                                    );
                                }
                                Err(error) => log(
                                    hilog::LOG_ERROR,
                                    format!("GPUI window open failed: {error}"),
                                ),
                            }
                        }
                    }
                }
                SurfaceEvent::Destroyed(_) => {
                    if runtime.window_generation == Some(generation) {
                        runtime.window_generation = None;
                        if let Some(handle) = runtime.window.take() {
                            if let Err(error) = runtime.app.update(|cx| {
                                handle.update(cx, |_, window, _| window.remove_window())
                            }) {
                                log(
                                    hilog::LOG_WARN,
                                    format!("GPUI window close failed: {error}"),
                                );
                            }
                        }
                    }
                }
            }
        });
        events += 1;
    }
    events + dispatcher.drain() as u32
}

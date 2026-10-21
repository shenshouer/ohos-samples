//! GPUI task dispatch. Foreground work is explicitly drained by the ArkUI host.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::Duration;

use crate::gpui_core::{PlatformDispatcher, Priority, RunnableVariant};

pub(crate) struct Dispatcher {
    main_thread: ThreadId,
    main_queue: Arc<Mutex<VecDeque<RunnableVariant>>>,
}

impl Dispatcher {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            main_thread: thread::current().id(),
            main_queue: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// Run queued foreground tasks only after the host callback has returned.
    pub(crate) fn drain(&self) -> usize {
        if !self.is_main_thread() {
            return 0;
        }
        let mut count = 0;
        // A bound prevents a self-scheduling task from trapping the ArkUI thread.
        while count < 256 {
            let runnable = self.main_queue.lock().ok().and_then(|mut q| q.pop_front());
            let Some(runnable) = runnable else { break };
            runnable.run();
            count += 1;
        }
        count
    }

    pub(crate) fn on_main_thread(&self) -> bool {
        self.is_main_thread()
    }
}

impl PlatformDispatcher for Dispatcher {
    fn is_main_thread(&self) -> bool {
        thread::current().id() == self.main_thread
    }

    fn dispatch(&self, runnable: RunnableVariant, _priority: Priority) {
        let _ = thread::Builder::new()
            .name("gpui-ohos-bg".into())
            .spawn(move || runnable.run());
    }

    fn dispatch_on_main_thread(&self, runnable: RunnableVariant, _priority: Priority) {
        if let Ok(mut queue) = self.main_queue.lock() {
            queue.push_back(runnable);
        }
    }

    fn dispatch_after(&self, duration: Duration, runnable: RunnableVariant) {
        let _ = thread::Builder::new()
            .name("gpui-ohos-timer".into())
            .spawn(move || {
                thread::sleep(duration);
                runnable.run();
            });
    }

    fn spawn_realtime(&self, f: Box<dyn FnOnce() + Send>) {
        let _ = thread::Builder::new()
            .name("gpui-ohos-realtime".into())
            .spawn(f);
    }
}

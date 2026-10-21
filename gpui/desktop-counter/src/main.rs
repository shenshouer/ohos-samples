use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};

struct Counter {
    value: i64,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(rgb(0xf4f6f8))
            .text_color(rgb(0x20242a))
            .child(div().text_xl().child("GPUI Counter"))
            .child(div().text_3xl().child(self.value.to_string()))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .id("decrement")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0xdce5ef))
                            .hover(|button| button.bg(rgb(0xcbd9e8)))
                            .cursor_pointer()
                            .child("−")
                            .on_click(cx.listener(|counter, _, _, cx| {
                                counter.value = counter.value.saturating_sub(1);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("increment")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x2d6cdf))
                            .text_color(rgb(0xffffff))
                            .hover(|button| button.bg(rgb(0x1d56bc)))
                            .cursor_pointer()
                            .child("+")
                            .on_click(cx.listener(|counter, _, _, cx| {
                                counter.value = counter.value.saturating_add(1);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("reset")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0xdce5ef))
                            .hover(|button| button.bg(rgb(0xcbd9e8)))
                            .cursor_pointer()
                            .child("Reset")
                            .on_click(cx.listener(|counter, _, _, cx| {
                                counter.value = 0;
                                cx.notify();
                            })),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(360.), px(240.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Counter { value: 0 }),
        )
        .expect("failed to open GPUI window");
        cx.activate(true);
    });
}

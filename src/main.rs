use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, Theme, TitleBar, WindowExt as _, v_flex,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    switch::Switch,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, TitlebarOptions, Window, WindowOptions, div,
};

const TITLE: &str = "GPUI Kit App";

struct App {
    name: Entity<InputState>,
    greeting: SharedString,
    count: usize,
    shout: bool,
    _subscriptions: Vec<Subscription>,
}

impl App {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Your name"));
        let subscription = cx.subscribe_in(&name, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.greeting = state.read(cx).value().to_string().into();
                cx.notify();
            }
        });


        // Follow the system light/dark setting, now and whenever it changes.
        Theme::sync_system_appearance(Some(window), cx);
        let appearance = cx.observe_window_appearance(window, |_, window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
            cx.notify();
        });

        Self {
            name,
            greeting: "".into(),
            count: 0,
            shout: false,
            _subscriptions: vec![subscription, appearance],
        }
    }

    fn greeting_text(&self) -> String {
        let who = if self.greeting.is_empty() {
            "World"
        } else {
            self.greeting.as_ref()
        };
        let text = format!("Hello, {who}!");
        if self.shout { text.to_uppercase() } else { text }
    }
}

impl Render for App {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = div()
            .flex()
            .flex_col()
            .flex_1()
            .p_6()
            .gap_4()
            .items_center()
            .justify_center()
            .child(div().text_2xl().child(self.greeting_text()))
            .child(div().w_64().child(Input::new(&self.name)))
            .child(
                Switch::new("shout")
                    .label("Shout")
                    .checked(self.shout)
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.shout = *value;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("increment")
                            .primary()
                            .icon(IconName::Plus)
                            .label(format!("Clicked {} times", self.count))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.count += 1;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("reset")
                            .small()
                            .label("Reset")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.count = 0;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("about")
                            .small()
                            .icon(IconName::Info)
                            .label("About")
                            .on_click(|_, window, cx| {
                                window.open_dialog(cx, |dialog, _, _| {
                                    dialog.title("About").child("A simple GPUI Kit app.")
                                });
                            }),
                    ),
            );

        // The compositor draws no window frame here, so render our own title bar.
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().child(TITLE))
            .child(body)
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            // Must be called before using any GPUI Kit component.
            gpui_kit::init(cx);

            // Opens a window whose content is wrapped in a `Root`, so dialogs,
            // sheets and notifications work.
            let options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(TITLE.into()),
                    ..TitleBar::title_bar_options()
                }),
                ..TitleBar::window_options()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| App::new(window, cx))
            })
            .expect("failed to open window");
        });
}

use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, Theme, ThemeMode, ThemeRegistry, TitleBar,
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    switch::Switch,
    v_flex,
};
use gpui_kit::{
    AppContext as _, AsyncApp, Context, Entity, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, TitlebarOptions, Window, WindowBounds, WindowOptions,
    div,
};

use std::time::{Duration, Instant};

mod window_state;

use window_state::WindowState;

const MACOS_CLASSIC_THEMES: &str = include_str!("../themes/macos-classic.json");

/// How long to wait for system light/dark events to settle before applying the theme.
const APPEARANCE_SETTLE: Duration = Duration::from_millis(50);

/// On Linux the system light/dark mode arrives asynchronously after startup, and GPUI reports
/// "light" until then. Wait up to this long for "dark" before opening the window.
const APPEARANCE_WAIT: Duration = Duration::from_millis(100);
const APPEARANCE_POLL: Duration = Duration::from_millis(5);

const TITLE: &str = "GPUI Kit App";

struct App {
    name: Entity<InputState>,
    greeting: SharedString,
    count: usize,
    shout: bool,
    window_bounds: WindowBounds,
    appearance_event: u64,
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

        // Follow the system light/dark setting when it changes. GPUI reports a bogus "light" first and the real mode a moment later, and applying the
        // bogus one repaints the window light. Wait for the events to settle, then apply the last.
        let appearance = cx.observe_window_appearance(window, |this, window, cx| {
            this.appearance_event += 1;
            let event = this.appearance_event;
            cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(APPEARANCE_SETTLE).await;
                this.update_in(cx, |this, window, cx| {
                    if this.appearance_event == event {
                        apply_system_theme(window, cx);
                        cx.notify();
                    }
                })
                .ok();
            })
            .detach();
        });

        // Keep the latest bounds and write them once, when the window goes away.
        let bounds = cx.observe_window_bounds(window, |this, window, _| {
            this.window_bounds = window.window_bounds();
        });
        cx.on_release(|this, _| WindowState::from_window_bounds(this.window_bounds).save())
            .detach();

        Self {
            name,
            greeting: "".into(),
            count: 0,
            shout: false,
            window_bounds: window.window_bounds(),
            appearance_event: 0,
            _subscriptions: vec![subscription, appearance, bounds],
        }
    }

    fn greeting_text(&self) -> String {
        let who = if self.greeting.is_empty() {
            "World"
        } else {
            self.greeting.as_ref()
        };
        let text = format!("Hello, {who}!");
        if self.shout {
            text.to_uppercase()
        } else {
            text
        }
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

/// Picks light or dark from the system setting, then installs the matching macOS Classic theme.
fn apply_system_theme(window: &mut Window, cx: &mut gpui_kit::App) {
    Theme::sync_system_appearance(Some(window), cx);
    apply_theme(cx.theme().mode.is_dark(), cx);
}

/// Installs the macOS Classic light or dark theme.
fn apply_theme(dark: bool, cx: &mut gpui_kit::App) {
    Theme::change(
        if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );

    let name = if dark {
        "macOS Classic Dark"
    } else {
        "macOS Classic Light"
    };
    let theme = ThemeRegistry::global(cx).themes().get(name).cloned();
    if let Some(theme) = theme {
        Theme::update(cx, |current| current.apply_config(&theme));
    }
}

/// Reads the system light/dark mode. On Linux it only becomes known shortly after startup, so
/// poll until it reports dark or the wait runs out; "light" cannot be told apart from "not yet
/// known", so a light system always waits the full time.
async fn system_is_dark(cx: &mut AsyncApp) -> bool {
    let is_dark =
        |cx: &mut AsyncApp| ThemeMode::from(cx.update(|cx| cx.window_appearance())).is_dark();

    if cfg!(target_os = "linux") {
        let deadline = Instant::now() + APPEARANCE_WAIT;
        while Instant::now() < deadline {
            if is_dark(cx) {
                return true;
            }
            cx.background_executor().timer(APPEARANCE_POLL).await;
        }
    }
    is_dark(cx)
}

/// Opens the main window. `Root` wraps its content, so dialogs, sheets and notifications work.
fn open_main_window(cx: &mut gpui_kit::App) {
    let saved = WindowState::load();
    let options = WindowOptions {
        window_bounds: saved.map(|state| state.to_window_bounds(cx)),
        titlebar: Some(TitlebarOptions {
            title: Some(TITLE.into()),
            ..TitleBar::title_bar_options()
        }),
        ..TitleBar::window_options()
    };
    gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| App::new(window, cx)))
        .expect("failed to open window");
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            // Must be called before using any GPUI Kit component.
            gpui_kit::init(cx);
            ThemeRegistry::global_mut(cx)
                .load_themes_from_str(MACOS_CLASSIC_THEMES)
                .expect("failed to load macOS Classic themes");

            // Open the window once the system mode is known, so it starts in the right theme.
            cx.spawn(async move |cx| {
                let dark = system_is_dark(cx).await;
                cx.update(|cx| {
                    apply_theme(dark, cx);
                    open_main_window(cx);
                });
            })
            .detach();
        });
}

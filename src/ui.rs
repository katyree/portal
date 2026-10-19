use crate::{
    catalog::{Catalog, LauncherId, LauncherStatus},
    search_input::TextInput,
};
use gpui::{
    AssetSource, Context, Entity, ImageSource, Resource, SharedString, Window, div, img,
    prelude::*, px, rgb, svg,
};
use std::{borrow::Cow, time::Duration};

pub struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let bytes: &[u8] = match path {
            "portal.png" => include_bytes!("../assets/portal.png"),
            "steam.svg" => include_bytes!("../assets/steam.svg"),
            "ubisoft.svg" => include_bytes!("../assets/ubisoft.svg"),
            "epicgames.svg" => include_bytes!("../assets/epicgames.svg"),
            "xbox.svg" => include_bytes!("../assets/xbox.svg"),
            "ea.svg" => include_bytes!("../assets/ea.svg"),
            "rockstargames.svg" => include_bytes!("../assets/rockstargames.svg"),
            "battledotnet.svg" => include_bytes!("../assets/battledotnet.svg"),
            "amazon.svg" => include_bytes!("../assets/amazon.svg"),
            "goggalaxy.svg" => include_bytes!("../assets/goggalaxy.svg"),
            "riot.svg" => include_bytes!("../assets/riot.svg"),
            "itch.svg" => include_bytes!("../assets/itch.svg"),
            "gamejolt.svg" => include_bytes!("../assets/gamejolt.svg"),
            "indiegala.svg" => include_bytes!("../assets/indiegala.svg"),
            "microsoftstore.svg" => include_bytes!("../assets/microsoftstore.svg"),
            "hytale.svg" => include_bytes!("../assets/hytale.svg"),
            "clonehero.svg" => include_bytes!("../assets/clonehero.svg"),
            "yarc.svg" => include_bytes!("../assets/yarc.svg"),
            _ => return Ok(None),
        };
        Ok(Some(Cow::Borrowed(bytes)))
    }
    fn list(&self, _: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

pub struct Portal {
    catalog: Option<Catalog>,
    launchers: Vec<LauncherStatus>,
    search: Entity<TextInput>,
    message: Option<String>,
    scan_generation: u64,
    settings_open: bool,
    installed_only: bool,
}

impl Portal {
    pub fn new(cx: &mut Context<Self>, settings_open: bool) -> Self {
        let search = cx.new(TextInput::new);
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        let (catalog, message) = match Catalog::load() {
            Ok(c) => (Some(c), None),
            Err(e) => (None, Some(e.to_string())),
        };
        let mut portal = Self {
            catalog,
            launchers: LauncherId::ALL
                .into_iter()
                .map(|id| LauncherStatus {
                    id,
                    target: None,
                    running: false,
                })
                .collect(),
            search,
            message,
            scan_generation: 0,
            settings_open,
            installed_only: false,
        };
        portal.refresh(cx);
        cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(5)).await;
                if view.update(cx, |portal, cx| portal.refresh(cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
        portal
    }
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(catalog) = self.catalog.clone() else {
            return;
        };
        self.scan_generation += 1;
        let generation = self.scan_generation;
        let scan = cx
            .background_executor()
            .spawn(async move { catalog.scan() });
        cx.spawn(async move |view, cx| {
            let launchers = scan.await;
            let _ = view.update(cx, |portal, cx| {
                if portal.scan_generation == generation {
                    portal.launchers = launchers;
                    cx.notify();
                }
            });
        })
        .detach();
    }
    fn toggle_integration(&mut self, id: LauncherId, cx: &mut Context<Self>) {
        if let Some(catalog) = &mut self.catalog {
            self.message = catalog
                .set_enabled(id, !catalog.is_enabled(id))
                .err()
                .map(|error| error.to_string());
            self.refresh(cx);
            cx.notify();
        }
    }
    fn launch(&mut self, id: LauncherId, cx: &mut Context<Self>) {
        let Some(catalog) = self.catalog.clone() else {
            return;
        };
        let task = cx
            .background_executor()
            .spawn(async move { catalog.launch(id) });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |portal, cx| {
                portal.message = result.err().map(|e| e.to_string());
                portal.refresh(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for Portal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content_width = (f32::from(window.viewport_size().width) - 72.).min(1600.);
        let columns = ((content_width + 16.) / (260. + 16.)).floor().clamp(1., 5.) as u16;
        let query = self.search.read(cx).content.to_lowercase();
        let cards: Vec<_> = self
            .launchers
            .iter()
            .filter(|l| self.catalog.as_ref().is_none_or(|c| c.is_enabled(l.id)))
            .filter(|l| !self.installed_only || l.target.is_some())
            .filter(|l| l.id.name().to_lowercase().contains(&query))
            .map(|launcher| {
                let id = launcher.id;
                let installed = launcher.target.is_some();
                let running = installed && launcher.running;
                div()
                    .id(id.slug())
                    .flex()
                    .flex_col()
                    .justify_between()
                    .gap(px(16.))
                    .h(px(136.))
                    .min_w_0()
                    .flex_shrink_0()
                    .p(px(18.))
                    .bg(rgb(0x222229))
                    .border_1()
                    .border_color(rgb(0x2e2e37))
                    .rounded(px(13.))
                    .hover(|style| style.border_color(rgb(0x655773)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .size(px(39.))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(11.))
                                    .bg(rgb(0x303039))
                                    .child(
                                        svg()
                                            .path(format!("{}.svg", id.slug()))
                                            .size(px(23.))
                                            .text_color(rgb(0xf5f4fa)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(rgb(if installed { 0xf0eff5 } else { 0x9997a3 }))
                                    .child(id.name()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_size(px(12.))
                                    .text_color(rgb(if running { 0x83cda0 } else { 0xa39eac }))
                                    .when(running, |d| {
                                        d.child(div().size(px(5.)).rounded_full().bg(rgb(0x83cda0)))
                                    })
                                    .child(if !installed {
                                        "Not installed"
                                    } else if running {
                                        "Running"
                                    } else {
                                        "Ready to launch"
                                    }),
                            )
                            .when(installed, |d| {
                                d.child(
                                    div()
                                        .id(("launch", id as usize))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .h(px(33.))
                                        .px(px(12.))
                                        .flex_shrink_0()
                                        .rounded(px(8.))
                                        .bg(rgb(0x373440))
                                        .hover(|style| style.bg(rgb(0x494052)))
                                        .text_color(rgb(0xd9cdee))
                                        .text_size(px(12.))
                                        .child(if running { "Open ↗" } else { "Launch ↗" })
                                        .on_click(cx.listener(move |portal, _, _, cx| {
                                            portal.launch(id, cx);
                                        })),
                                )
                            }),
                    )
            })
            .collect();
        let empty = cards.is_empty();
        let settings_rows = LauncherId::ALL
            .into_iter()
            .map(|id| {
                let enabled = self.catalog.as_ref().is_some_and(|c| c.is_enabled(id));
                div()
                    .flex()
                    .items_center()
                    .gap(px(16.))
                    .h(px(65.))
                    .flex_shrink_0()
                    .px(px(17.))
                    .bg(rgb(0x222229))
                    .border_1()
                    .border_color(rgb(0x2e2e37))
                    .rounded(px(13.))
                    .child(
                        svg()
                            .path(format!("{}.svg", id.slug()))
                            .size(px(23.))
                            .text_color(rgb(0xf5f4fa)),
                    )
                    .child(div().flex_1().text_size(px(13.)).child(id.name()))
                    .child(
                        div()
                            .id(("integration", id as usize))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .when(enabled, |d| d.justify_end())
                            .w(px(40.))
                            .h(px(24.))
                            .px(px(3.))
                            .rounded_full()
                            .bg(rgb(if enabled { 0xbaabde } else { 0x45434e }))
                            .child(div().size(px(18.)).rounded_full().bg(rgb(if enabled {
                                0x28232f
                            } else {
                                0xb3afbd
                            })))
                            .on_click(cx.listener(move |portal, _, _, cx| {
                                portal.toggle_integration(id, cx)
                            })),
                    )
            })
            .collect::<Vec<_>>();
        div()
            .size_full()
            .bg(rgb(0x18181e))
            .text_color(rgb(0xf0eff5))
            .font_family("Segoe UI")
            .flex()
            .flex_col()
            .items_center()
            .px(px(36.))
            .py(px(29.))
            .gap(px(26.))
            .child(
                div()
                    .w_full()
                    .max_w(px(1600.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(9.))
                            .child(
                                img(ImageSource::Resource(Resource::Embedded(
                                    "portal.png".into(),
                                )))
                                .size(px(24.)),
                            )
                            .child(div().text_size(px(12.)).child("portal")),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(18.))
                            .when(!self.settings_open, |d| {
                                d.child(
                                    div()
                                        .id("rescan")
                                        .cursor_pointer()
                                        .text_size(px(19.))
                                        .text_color(rgb(0x96919f))
                                        .child("↻")
                                        .on_click(
                                            cx.listener(|portal, _, _, cx| portal.refresh(cx)),
                                        ),
                                )
                            })
                            .child(
                                div()
                                    .id("settings")
                                    .cursor_pointer()
                                    .text_color(rgb(0x96919f))
                                    .text_size(px(12.))
                                    .child(if self.settings_open {
                                        "← Back"
                                    } else {
                                        "Settings"
                                    })
                                    .on_click(cx.listener(|portal, _, _, cx| {
                                        portal.settings_open = !portal.settings_open;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .max_w(px(1600.))
                    .flex_shrink_0()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(18.))
                    .child(div().text_size(px(29.)).child(if self.settings_open {
                        "Settings"
                    } else {
                        "Launchers"
                    }))
                    .when(!self.settings_open, |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(14.))
                                .child(
                                    div()
                                        .id("installed-only")
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .text_size(px(12.))
                                        .text_color(rgb(0xc4bfce))
                                        .child("Installed only")
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .when(self.installed_only, |d| d.justify_end())
                                                .w(px(32.))
                                                .h(px(20.))
                                                .px(px(3.))
                                                .rounded_full()
                                                .bg(rgb(if self.installed_only {
                                                    0xbaabde
                                                } else {
                                                    0x45434e
                                                }))
                                                .child(div().size(px(14.)).rounded_full().bg(rgb(
                                                    if self.installed_only {
                                                        0x28232f
                                                    } else {
                                                        0xb3afbd
                                                    },
                                                ))),
                                        )
                                        .on_click(cx.listener(|portal, _, _, cx| {
                                            portal.installed_only = !portal.installed_only;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    div()
                                        .w(px(173.))
                                        .rounded(px(8.))
                                        .overflow_hidden()
                                        .child(self.search.clone()),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .id("launcher-list")
                    .w_full()
                    .max_w(px(1600.))
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .overflow_y_scroll()
                    .flex_1()
                    .when(self.settings_open, |d| d.children(settings_rows))
                    .when(!self.settings_open && !empty, |d| {
                        d.child(
                            div()
                                .grid()
                                .grid_cols(columns)
                                .gap(px(16.))
                                .flex_shrink_0()
                                .children(cards),
                        )
                    })
                    .when(!self.settings_open && empty, |d| {
                        d.child(div().text_color(rgb(0x96919f)).text_size(px(13.)).child(
                            if self.catalog.as_ref().is_some_and(|c| {
                                LauncherId::ALL.into_iter().all(|id| !c.is_enabled(id))
                            }) {
                                "Enable launchers in Settings"
                            } else if self.installed_only {
                                "No installed launchers found"
                            } else {
                                "No launchers found"
                            },
                        ))
                    }),
            )
            .when_some(self.message.clone(), |d, message| {
                d.child(
                    div()
                        .w_full()
                        .max_w(px(1600.))
                        .flex_shrink_0()
                        .text_size(px(12.))
                        .text_color(rgb(0xe4a4a5))
                        .child(message),
                )
            })
    }
}

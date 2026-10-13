use crate::{
    catalog::{Catalog, LauncherId, LauncherStatus},
    search_input::TextInput,
};
use gpui::{
    AssetSource, Context, Entity, PathPromptOptions, SharedString, Window, div, prelude::*, px,
    rgb, svg,
};
use std::{borrow::Cow, time::Duration};

pub struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let bytes: &[u8] = match path {
            "steam.svg" => include_bytes!("../assets/steam.svg"),
            "ubisoft.svg" => include_bytes!("../assets/ubisoft.svg"),
            "epicgames.svg" => include_bytes!("../assets/epicgames.svg"),
            "xbox.svg" => include_bytes!("../assets/xbox.svg"),
            "ea.svg" => include_bytes!("../assets/ea.svg"),
            "rockstargames.svg" => include_bytes!("../assets/rockstargames.svg"),
            "battledotnet.svg" => include_bytes!("../assets/battledotnet.svg"),
            "amazon.svg" => include_bytes!("../assets/amazon.svg"),
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
}

impl Portal {
    pub fn new(cx: &mut Context<Self>) -> Self {
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
    fn locate(&mut self, id: LauncherId, cx: &mut Context<Self>) {
        if self.catalog.is_none() {
            return;
        }
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(format!("Locate {} executable", id.name()).into()),
        });
        cx.spawn(async move |view, cx| {
            let selected = match picker.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) => return,
                _ => {
                    let _ = view.update(cx, |portal, cx| {
                        portal.message = Some("Could not open the file picker.".into());
                        cx.notify();
                    });
                    return;
                }
            };
            if let Some(path) = selected {
                let _ = view.update(cx, |portal, cx| {
                    if let Some(catalog) = &mut portal.catalog {
                        portal.message = catalog
                            .set_executable(id, path)
                            .err()
                            .map(|e| e.to_string());
                        portal.refresh(cx);
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }
}

impl Render for Portal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search.read(cx).content.to_lowercase();
        let rows: Vec<_> = self
            .launchers
            .iter()
            .filter(|l| l.id.name().to_lowercase().contains(&query))
            .map(|launcher| {
                let id = launcher.id;
                let installed = launcher.target.is_some();
                div()
                    .id(id.slug())
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
                        div()
                            .size(px(39.))
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
                            .flex_1()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(13.))
                            .text_color(rgb(if installed { 0xf0eff5 } else { 0x9997a3 }))
                            .child(id.name()),
                    )
                    .child(
                        div()
                            .id(("locate", id as usize))
                            .cursor_pointer()
                            .text_size(px(16.))
                            .text_color(rgb(0x9f9baa))
                            .w(px(26.))
                            .text_center()
                            .child("⌑")
                            .on_click(cx.listener(move |portal, _, _, cx| portal.locate(id, cx))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap(px(6.))
                            .w(px(61.))
                            .flex_shrink_0()
                            .text_size(px(11.))
                            .text_color(rgb(if launcher.running { 0x83cda0 } else { 0xa39eac }))
                            .when(launcher.running, |d| {
                                d.child(div().size(px(5.)).rounded_full().bg(rgb(0x83cda0)))
                            })
                            .child(if launcher.running {
                                "Running"
                            } else {
                                "Launch"
                            }),
                    )
                    .child(
                        div()
                            .id(("launch", id as usize))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(33.))
                            .rounded(px(9.))
                            .bg(rgb(if installed { 0x373440 } else { 0x292930 }))
                            .text_color(rgb(if installed { 0xd9cdee } else { 0x817e8b }))
                            .text_size(px(18.))
                            .child(if installed { "↗" } else { "+" })
                            .on_click(cx.listener(move |portal, _, _, cx| {
                                if installed {
                                    portal.launch(id, cx);
                                } else {
                                    portal.locate(id, cx);
                                }
                            })),
                    )
            })
            .collect();
        div()
            .size_full()
            .bg(rgb(0x18181e))
            .text_color(rgb(0xf0eff5))
            .font_family("Segoe UI")
            .flex()
            .flex_col()
            .px(px(36.))
            .py(px(29.))
            .gap(px(26.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(9.))
                            .child(div().size(px(18.)).rounded(px(6.)).bg(rgb(0xbaabde)))
                            .child(div().text_size(px(12.)).child("portal")),
                    )
                    .child(
                        div()
                            .id("rescan")
                            .cursor_pointer()
                            .text_color(rgb(0x96919f))
                            .text_size(px(19.))
                            .child("↻")
                            .on_click(cx.listener(|portal, _, _, cx| portal.refresh(cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(18.))
                    .child(div().text_size(px(29.)).child("Launchers"))
                    .child(
                        div()
                            .w(px(173.))
                            .rounded(px(8.))
                            .overflow_hidden()
                            .child(self.search.clone()),
                    ),
            )
            .child(
                div()
                    .id("launcher-list")
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .overflow_y_scroll()
                    .flex_1()
                    .children(rows)
                    .when(
                        self.launchers
                            .iter()
                            .all(|l| !l.id.name().to_lowercase().contains(&query)),
                        |d| {
                            d.child(
                                div()
                                    .text_color(rgb(0x96919f))
                                    .text_size(px(13.))
                                    .child("No launchers found"),
                            )
                        },
                    ),
            )
            .when_some(self.message.clone(), |d, message| {
                d.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0xe4a4a5))
                        .child(message),
                )
            })
    }
}

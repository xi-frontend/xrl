use crate::api::{CharRef, LineRef, StyleCache, View, ViewList};
use crate::protocol::{Message, Plugin, Request, ThemeChanged, ViewId, XiNotification};

#[derive(Default)]
pub struct Editor {
    pub styles: StyleCache,
    pub views: ViewList<View>,
    pub languages: Vec<String>,
    pub requests: Vec<Request>,
    pub plugins: Vec<Plugin>,
    pub themes: Vec<String>,
    pub theme: Option<ThemeChanged>,
}

impl Editor {
    pub fn with<F: FnOnce(&mut View)>(&mut self, view: Option<ViewId>, func: F) {
        let view = if let Some(view) = view {
            self.views.get_mut(&view)
        } else {
            self.views.get_current_mut()
        };
        if let Some(mut view) = view {
            func(&mut view);
        }
    }

    pub fn new_view(&mut self, id: ViewId) {
        self.views.add(id, View::new(id));
    }

    pub fn xi_message(&mut self, msg: Message) {
        match msg {
            Message::Notification(note) => self.xi_notification(note),
            Message::Request(req) => self.requests.push(req),
            Message::Response(res) => {
                for req in &self.requests {
                    if req.id == res.id && req.method == "new_view"{
                        if let Ok(value) = &res.result {
                            if let Some(view_id) = value.get("view_id") {
                                if let Some(num) = view_id.as_u64() {
                                    self.views.add(
                                        ViewId::from(num as usize),
                                        View::new(ViewId::from(num as usize)),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Message::Error(_err) => {}
        }
    }

    pub fn xi_notification(&mut self, msg: XiNotification) {
        match msg {
            XiNotification::Update(update) => {
                self.with(Some(update.view_id), |view| view.update(update))
            }
            XiNotification::DefStyle(style) => self.styles.insert(style.id, style),
            XiNotification::AvailablePlugins(plugins) => self.plugins = plugins.plugins,
            XiNotification::AvailableThemes(themes) => self.themes = themes.themes,
            XiNotification::AvailableLanguages(langs) => self.languages = langs.languages,
            XiNotification::ThemeChanged(theme) => self.theme = Some(theme),
            XiNotification::ConfigChanged(conf) => {
                self.with(Some(conf.view_id), |view| view.config = Some(conf.changes))
            }
            XiNotification::LanguageChanged(lang) => self.with(Some(lang.view_id), |view| {
                view.language = Some(lang.language_id)
            }),
            XiNotification::PluginStarted(plugin) => self.with(Some(plugin.view_id), |view| {
                view.plugins.push(plugin.plugin)
            }),
            XiNotification::PluginStoped(plugin) => self.with(Some(plugin.view_id), |view| {
                view.plugins.retain(|item| item != &plugin.plugin)
            }),
            XiNotification::FindStatus(find) => {
                self.with(Some(find.view_id), |view| view.find_status = Some(find))
            }
            XiNotification::ReplaceStatus(status) => self.with(Some(status.view_id), |view| {
                view.replace_status = Some(status.status)
            }),
            XiNotification::ScrollTo(scroll) => {
                self.with(None, |view| view.viewport.scroll_to(scroll))
            }
            XiNotification::UpdateCmds(_) => {}
            XiNotification::Alert(_) => {}
        }
    }

    pub fn render_lines(&self) -> Option<impl Iterator<Item = LineRef<'_>>> {
        if let Some(view) = self.views.get_current() {
            Some(view.render_lines())
        } else {
            None
        }
    }

    pub fn render_chars(
        &self,
    ) -> Option<impl Iterator<Item = impl Iterator<Item = CharRef> + '_>> {
        if let Some(view) = self.views.get_current() {
            Some(view.render_chars())
        } else {
            None
        }
    }
}

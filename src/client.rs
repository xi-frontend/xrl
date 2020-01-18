use crate::errors::ClientError;
use crate::protocol;
use crate::structs::{ModifySelection, ViewId};
use futures::{future, future::Either, Future};
use serde::Serialize;
use serde_json::Value;
use serde_json::{from_value, to_value, Map};

/// A client to send notifications and request to xi-core.
#[derive(Clone)]
pub struct Client(pub protocol::Client);

fn get_edit_params<T: Serialize>(
    view_id: ViewId,
    method: &str,
    params: Option<T>,
) -> Result<Value, ClientError> {
    let params_value = if let Some(params) = params {
        to_value(params)?
    } else {
        json!([])
    };

    Ok(json!({
        "method": method,
        "view_id": view_id,
        "params": params_value,
    }))
}

impl Client {
    /// Send a notification to the core. Most (if not all) notifications
    /// supported by the core are already implemented, so this method
    /// should not be necessary in most cases.
    pub fn notify(
        &self,
        method: &str,
        params: Value,
    ) -> impl Future<Item = (), Error = ClientError> {
        info!(">>> notification: method={}, params={}", method, &params);
        self.0
            .notify(method, params)
            .map_err(|_| ClientError::NotifyFailed)
    }

    /// Send a request to the core. Most (if not all) notifications
    /// supported by the core are already implemented, so this method
    /// should not be necessary in most cases.
    pub fn request(
        &self,
        method: &str,
        params: Value,
    ) -> impl Future<Item = Value, Error = ClientError> {
        info!(">>> request : method={}, params={}", method, &params);
        self.0
            .request(method, params)
            .then(|response| match response {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(value)) => Err(ClientError::ErrorReturned(value)),
                Err(_) => Err(ClientError::RequestFailed),
            })
    }

    pub fn edit_request<T: Serialize>(
        &self,
        view_id: ViewId,
        method: &str,
        params: Option<T>,
    ) -> impl Future<Item = Value, Error = ClientError> {
        match get_edit_params(view_id, method, params) {
            Ok(value) => Either::A(self.request("edit", value)),
            Err(e) => Either::B(future::err(e)),
        }
    }

    /// Send an "edit" notification. Most (if not all) "edit" commands are
    /// already implemented, so this method should not be necessary in most
    /// cases.
    pub fn edit_notify<T: Serialize>(
        &self,
        view_id: ViewId,
        method: &str,
        params: Option<T>,
    ) -> impl Future<Item = (), Error = ClientError> {
        match get_edit_params(view_id, method, params) {
            Ok(value) => Either::A(self.notify("edit", value)),
            Err(e) => Either::B(future::err(e)),
        }
    }

    /// Send an "scroll" notification
    /// ```ignore
    /// {"method":"edit","params":{"method":"scroll","params":[21,80],
    /// "view_id":"view-id-1"}}
    /// ```
    pub fn scroll(
        &self,
        view_id: ViewId,
        first_line: u64,
        last_line: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "scroll", Some(json!([first_line, last_line])))
    }

    pub fn goto_line(
        &self,
        view_id: ViewId,
        line: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "goto_line", Some(json!({ "line": line })))
    }

    pub fn copy(&self, view_id: ViewId) -> impl Future<Item = Value, Error = ClientError> {
        self.edit_request(view_id, "copy", None as Option<Value>)
    }

    pub fn paste(
        &self,
        view_id: ViewId,
        buffer: &str,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "paste", Some(json!({ "chars": buffer })))
    }

    pub fn cut(&self, view_id: ViewId) -> impl Future<Item = Value, Error = ClientError> {
        self.edit_request(view_id, "cut", None as Option<Value>)
    }

    pub fn undo(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "undo", None as Option<Value>)
    }

    pub fn redo(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "redo", None as Option<Value>)
    }

    pub fn find(
        &self,
        view_id: ViewId,
        search_term: &str,
        case_sensitive: bool,
        regex: bool,
        whole_words: bool,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "find",
            Some(json!({
                "chars": search_term,
                "case_sensitive": case_sensitive,
                "regex": regex,
                "whole_words": whole_words})),
        )
    }

    fn find_other(
        &self,
        view_id: ViewId,
        command: &str,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            command,
            Some(json!({
                "wrap_around": wrap_around,
                "allow_same": allow_same,
                "modify_selection": modify_selection})),
        )
    }

    pub fn find_next(
        &self,
        view_id: ViewId,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.find_other(
            view_id,
            "find_next",
            wrap_around,
            allow_same,
            modify_selection,
        )
    }

    pub fn find_prev(
        &self,
        view_id: ViewId,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.find_other(
            view_id,
            "find_previous",
            wrap_around,
            allow_same,
            modify_selection,
        )
    }

    pub fn find_all(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "find_all", None as Option<Value>)
    }

    pub fn highlight_find(
        &self,
        view_id: ViewId,
        visible: bool,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "highlight_find",
            Some(json!({ "visible": visible })),
        )
    }

    pub fn left(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_left", None as Option<Value>)
    }

    pub fn left_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_left_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn right(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_right", None as Option<Value>)
    }

    pub fn right_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_right_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn up(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_up", None as Option<Value>)
    }

    pub fn up_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_up_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn down(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_down", None as Option<Value>)
    }

    pub fn down_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_down_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn backspace(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.del(view_id)
    }

    pub fn delete(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "delete_forward", None as Option<Value>)
    }

    pub fn del(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "delete_backward", None as Option<Value>)
    }

    pub fn delete_word_backward(
        &self,
        view_id: ViewId,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "delete_word_backward", None as Option<Value>)
    }

    pub fn page_up(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "scroll_page_up", None as Option<Value>)
    }

    pub fn page_up_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "page_up_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn page_down(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "scroll_page_down", None as Option<Value>)
    }

    pub fn page_down_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "page_down_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn line_start(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_to_left_end_of_line", None as Option<Value>)
    }

    pub fn line_start_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_to_left_end_of_line_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn line_end(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_to_right_end_of_line", None as Option<Value>)
    }

    pub fn line_end_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_to_right_end_of_line_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn document_begin(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_to_beginning_of_document",
            None as Option<Value>,
        )
    }

    pub fn document_begin_sel(
        &self,
        view_id: ViewId,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_to_beginning_of_document_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn document_end(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_to_end_of_document", None as Option<Value>)
    }

    pub fn document_end_sel(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_to_end_of_document_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn select_all(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "select_all", None as Option<Value>)
    }

    pub fn collapse_selections(
        &self,
        view_id: ViewId,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "collapse_selections", None as Option<Value>)
    }

    pub fn insert(
        &self,
        view_id: ViewId,
        string: &str,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "insert", Some(json!({ "chars": string })))
    }

    pub fn insert_newline(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "insert_newline", None as Option<Value>)
    }

    pub fn insert_tab(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "insert_tab", None as Option<Value>)
    }

    pub fn f1(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "debug_rewrap", None as Option<Value>)
    }

    pub fn f2(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "debug_test_fg_spans", None as Option<Value>)
    }

    pub fn char(&self, view_id: ViewId, ch: char) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "insert", Some(json!({ "chars": ch })))
    }

    // FIXME: handle modifier and click count
    pub fn click(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "click", Some(json!([line, column, 0, 1])))
    }

    pub fn click_point_select(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "point_select";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn click_toggle_sel(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "toggle_sel";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn click_range_select(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "range_select";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn click_line_select(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "range_select";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn click_word_select(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "word_select";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn click_multi_line_select(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "multi_line_select";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn click_multi_word_select(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        let ty = "multi_word_select";
        self.edit_notify(
            view_id,
            "gesture",
            Some(json!({"line": line, "col": column, "ty": ty,})),
        )
    }

    pub fn drag(
        &self,
        view_id: ViewId,
        line: u64,
        column: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "drag", Some(json!([line, column, 0])))
    }

    /// send a `"new_view"` request to the core.
    /// ```ignore
    /// {"id":1,"method":"new_view","params":{"file_path":"foo/test.txt"}}
    /// ```
    pub fn new_view(
        &self,
        file_path: Option<String>,
    ) -> impl Future<Item = ViewId, Error = ClientError> {
        let params = if let Some(file_path) = file_path {
            json!({ "file_path": file_path })
        } else {
            json!({})
        };
        self.request("new_view", params)
            .and_then(|result| from_value::<ViewId>(result).map_err(From::from))
    }

    /// send a `"close_view"` notifycation to the core.
    pub fn close_view(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.notify("close_view", json!({ "view_id": view_id }))
    }

    pub fn save(
        &self,
        view_id: ViewId,
        file_path: &str,
    ) -> impl Future<Item = (), Error = ClientError> {
        let params = json!({"view_id": view_id, "file_path": file_path});
        self.notify("save", params).and_then(|_| Ok(()))
    }

    pub fn set_theme(&self, theme: &str) -> impl Future<Item = (), Error = ClientError> {
        let params = json!({ "theme_name": theme });
        self.notify("set_theme", params).and_then(|_| Ok(()))
    }

    pub fn client_started(
        &self,
        config_dir: Option<&str>,
        client_extras_dir: Option<&str>,
    ) -> impl Future<Item = (), Error = ClientError> {
        let mut params = Map::new();
        if let Some(path) = config_dir {
            let _ = params.insert("config_dir".into(), json!(path));
        }
        if let Some(path) = client_extras_dir {
            let _ = params.insert("client_extras_dir".into(), json!(path));
        }
        self.notify("client_started", params.into())
    }

    pub fn start_plugin(
        &self,
        view_id: ViewId,
        name: &str,
    ) -> impl Future<Item = (), Error = ClientError> {
        let params = json!({"view_id": view_id, "plugin_name": name});
        self.notify("start", params).and_then(|_| Ok(()))
    }

    pub fn stop_plugin(
        &self,
        view_id: ViewId,
        name: &str,
    ) -> impl Future<Item = (), Error = ClientError> {
        let params = json!({"view_id": view_id, "plugin_name": name});
        self.notify("stop", params).and_then(|_| Ok(()))
    }

    pub fn notify_plugin(
        &self,
        view_id: ViewId,
        plugin: &str,
        method: &str,
        params: &Value,
    ) -> impl Future<Item = (), Error = ClientError> {
        let params = json!({
            "view_id": view_id,
            "receiver": plugin,
            "notification": {
                "method": method,
                "params": params,
            }
        });
        self.notify("plugin_rpc", params).and_then(|_| Ok(()))
    }

    pub fn outdent(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "outdent", None as Option<Value>)
    }

    pub fn move_word_left(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_word_left", None as Option<Value>)
    }

    pub fn move_word_right(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "move_word_right", None as Option<Value>)
    }

    pub fn move_word_left_sel(
        &self,
        view_id: ViewId,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_word_left_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn move_word_right_sel(
        &self,
        view_id: ViewId,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "move_word_right_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn resize(
        &self,
        view_id: ViewId,
        width: i32,
        height: i32,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "resize",
            Some(json!({
                "width": width,
                "height": height,
            })),
        )
    }

    pub fn replace(
        &self,
        view_id: ViewId,
        chars: &str,
        preserve_case: bool,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "replace",
            Some(json!({
                "chars": chars,
                "preserve_case": preserve_case,
            })),
        )
    }

    pub fn replace_next(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "replace_next", None as Option<Value>)
    }

    pub fn replace_all(&self, view_id: ViewId) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(view_id, "replace_all", None as Option<Value>)
    }

    pub fn set_language(
        &self,
        view_id: ViewId,
        lang_name: &str,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.notify(
            "set_language",
            json!({ "view_id": view_id, "language_id": lang_name }),
        )
    }

    pub fn selection_for_find(
        &self,
        view_id: ViewId,
        case_sensitive: bool,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.notify(
            "selection_for_find",
            json!({ "view_id": view_id, "case_sensitive": case_sensitive }),
        )
    }

    pub fn selection_for_replace(
        &self,
        view_id: ViewId,
        case_sensitive: bool,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.notify(
            "selection_for_replace",
            json!({ "view_id": view_id, "case_sensitive": case_sensitive }),
        )
    }

    pub fn selection_into_lines(
        &self,
        view_id: ViewId,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.notify("selection_into_lines", json!({ "view_id": view_id }))
    }

    //TODO: Use something more elegant than a `Value`
    pub fn modify_user_config(
        &self,
        domain: &str,
        changes: Value,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.notify(
            "modify_user_config",
            json!({
                "domain": domain,
                "changes": changes,
            }),
        )
    }

    pub fn request_lines(
        &self,
        view_id: ViewId,
        first_line: u64,
        last_line: u64,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.edit_notify(
            view_id,
            "request_lines",
            Some(json!([first_line, last_line])),
        )
    }

    pub fn shutdown(&self) {
        self.0.shutdown()
    }

    // TODO: requests for plugin_rpc
}

use futures::{future, Future};
use serde_json::Value;
use errors::ClientError;
use protocol;
use serde_json::{from_value, to_value};
use serde::Serialize;

pub type ClientResult<T> = Box<Future<Item = T, Error = ClientError>>;

#[derive(Clone)]
pub struct Client(pub protocol::Client);

fn get_edit_params<T: Serialize>(
    view_id: &str,
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
    /// Send a notification to the core
    pub fn notify(&mut self, method: &str, params: Value) -> ClientResult<()> {
        Box::new(
            self.0
                .notify(method, params)
                .map_err(|_| ClientError::NotifyFailed),
        )
    }

    pub fn request(&mut self, method: &str, params: Value) -> ClientResult<Value> {
        Box::new(self.0.request(method, params).then(
            |response| match response {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(value)) => Err(ClientError::ErrorReturned(value)),
                Err(_) => Err(ClientError::RequestFailed),
            },
        ))
    }

    /// Send an
    /// ["edit" notification](https://github.com/google/xi-editor/blob/c215deea8c2dfced91a9e019e1febdc8ce68158e/doc/frontend.md#edit)
    fn edit<T: Serialize>(
        &mut self,
        view_id: &str,
        method: &str,
        params: Option<T>,
    ) -> ClientResult<()> {
        match get_edit_params(view_id, method, params) {
            Ok(value) => self.notify("edit", value),
            Err(e) => Box::new(future::err(e)),
        }
    }

    pub fn scroll(&mut self, view_id: &str, first_line: u64, last_line: u64) -> ClientResult<()> {
        self.edit(view_id, "scroll", Some(json!([first_line, last_line])))
    }

    pub fn left(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "move_left", None as Option<Value>)
    }

    pub fn left_sel(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(
            view_id,
            "move_left_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn right(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "move_right", None as Option<Value>)
    }

    pub fn right_sel(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(
            view_id,
            "move_right_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn up(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "move_up", None as Option<Value>)
    }

    pub fn up_sel(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(
            view_id,
            "move_up_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn down(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "move_down", None as Option<Value>)
    }

    pub fn down_sel(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(
            view_id,
            "move_down_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn del(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "delete_backward", None as Option<Value>)
    }

    pub fn page_up(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "page_up", None as Option<Value>)
    }

    pub fn page_up_sel(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(
            view_id,
            "page_up_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn page_down(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "page_down", None as Option<Value>)
    }

    pub fn page_down_sel(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(
            view_id,
            "page_down_and_modify_selection",
            None as Option<Value>,
        )
    }

    pub fn insert_newline(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "insert_newline", None as Option<Value>)
    }

    pub fn f1(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "debug_rewrap", None as Option<Value>)
    }

    pub fn f2(&mut self, view_id: &str) -> ClientResult<()> {
        self.edit(view_id, "debug_test_fg_spans", None as Option<Value>)
    }

    pub fn char(&mut self, view_id: &str, ch: char) -> ClientResult<()> {
        self.edit(view_id, "insert", Some(json!({ "chars": ch })))
    }

    // FIXME: handle modifier and click count
    pub fn click(&mut self, view_id: &str, line: u64, column: u64) -> ClientResult<()> {
        self.edit(view_id, "click", Some(json!([line, column, 0, 1])))
    }

    /// Implements dragging (extending a selection). Arguments are line, column, and flag as in click.
    /// [Xi documentation](https://github.com/google/xi-editor/blob/c215deea8c2dfced91a9e019e1febdc8ce68158e/doc/frontend.md#drag)
    pub fn drag(&mut self, view_id: &str, line: u64, column: u64) -> ClientResult<()> {
        self.edit(view_id, "drag", Some(json!([line, column, 0])))
    }

    /// Creates a new view, returning the view identifier as a string. file_path is optional; if
    /// specified, the file is loaded into a new buffer; if not a new empty buffer is created.
    /// Currently, only a single view into a given file can be open at a time.
    ///
    /// Note, there is currently no mechanism for reporting errors. Also note, the protocol
    /// delegates power to load and save arbitrary files. Thus, exposing the protocol to any other
    /// agent than a front-end in direct control should be done with extreme caution.
    ///
    /// [Xi documentation](https://github.com/google/xi-editor/blob/c215deea8c2dfced91a9e019e1febdc8ce68158e/doc/frontend.md#new_view)
    pub fn new_view(&mut self, file_path: Option<String>) -> ClientResult<String> {
        let params = if let Some(file_path) = file_path {
            json!({ "file_path": file_path })
        } else {
            json!({})
        };
        let result = self.request("new_view", params)
            .and_then(|result| from_value::<String>(result).map_err(From::from));
        Box::new(result)
    }

    pub fn close_view(&mut self, view_id: &str) -> ClientResult<()> {
        self.notify("close_view", json!({ "view_id": view_id }))
    }

    pub fn save(&mut self, view_id: &str, file_path: &str) -> ClientResult<()> {
        let params = json!({"view_id": view_id, "file_path": file_path});
        Box::new(self.request("save", params).and_then(|_| Ok(())))
    }
}

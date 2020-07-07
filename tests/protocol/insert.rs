use serde_json::json;

use std::io;
use std::thread::sleep;
use std::time::Duration;

use xrl::client::ClientExt;
use xrl::protocol::{Annotation, Line, Message, Operation, OperationType, XiNotification};
use xrl::protocol::{Update, UpdateNotification, ViewId};
use xrl::TestClient;

#[tokio::test]
async fn insert_into_new_view() -> io::Result<()> {
    let mut client = TestClient::embeded().await?;

    client.new_view(None).await?;
    sleep(Duration::from_secs(1));

    let line = Line {
        text: "".into(),
        cursor: vec![0],
        styles: vec![],
        line_num: Some(1),
    };
    let operation = Operation {
        operation_type: OperationType::Insert,
        nb_lines: 1,
        line_num: None,
        lines: vec![line],
    };
    let annotation = Annotation {
        ty: "selection".into(),
        ranges: vec![[0, 0, 0, 0]],
        payloads: json!(null),
        n: 1,
    };
    let update = Update {
        annotations: vec![annotation],
        operations: vec![operation],
        pristine: true,
        rev: None,
    };
    let update = UpdateNotification {
        view_id: ViewId::from(1),
        update,
    };
    let expected = Message::Notification(XiNotification::Update(update));

    client.insert(ViewId::from(1), "data").await?;
    sleep(Duration::from_secs(1));

    client.check_responses(Some(10), expected).await?;
    Ok(())
}

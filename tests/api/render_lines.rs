use serde_json::json;

use xrl::api::{LineRef, View};
use xrl::protocol::{
    Annotation, Line, Operation, OperationType, StyleDef, Update, UpdateNotification, ViewId,
};

#[test]
fn simple() {
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

    let mut view = View::new(From::from(1));
    view.viewport.resize(10, 10);
    view.update(update);
    let rendered_lines: Vec<LineRef<'_>> = view.render_lines().collect();
    let line_ref = LineRef {
        text: "",
        styles: vec![],
        cursor: &[0],
        line_num: Some(1),
    };
    assert_eq!(vec![line_ref], rendered_lines);
}

#[test]
fn styled() {
    let line = Line {
        text: "some text".into(),
        cursor: vec![0],
        styles: vec![
            StyleDef {
                length: 4,
                offset: 0,
                style_id: 1,
            },
            StyleDef {
                length: 4,
                offset: 1,
                style_id: 3,
            },
        ],
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
        ranges: vec![],
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

    let mut view = View::new(From::from(1));
    view.viewport.resize(10, 10);
    view.update(update);
    let rendered_lines: Vec<LineRef<'_>> = view.render_lines().collect();
    let line_ref = LineRef {
        text: "some text",
        styles: vec![
            StyleDef {
                length: 4,
                offset: 0,
                style_id: 1,
            },
            StyleDef {
                length: 4,
                offset: 1,
                style_id: 3,
            },
        ],
        cursor: &[0],
        line_num: Some(1),
    };
    assert_eq!(vec![line_ref], rendered_lines);
}

#[test]
fn styled_offset() {
    let line = Line {
        text: "    some text".into(),
        cursor: vec![0],
        styles: vec![
            StyleDef {
                length: 4,
                offset: 4,
                style_id: 1,
            },
            StyleDef {
                length: 4,
                offset: 1,
                style_id: 3,
            },
        ],
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
        ranges: vec![],
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

    let mut view = View::new(From::from(1));
    view.viewport.resize(10, 10);
    view.viewport.horizontal_offset = 2;
    view.update(update);
    let rendered_lines: Vec<LineRef<'_>> = view.render_lines().collect();
    let line_ref = LineRef {
        text: "  some text",
        styles: vec![
            StyleDef {
                length: 4,
                offset: 2,
                style_id: 1,
            },
            StyleDef {
                length: 4,
                offset: 1,
                style_id: 3,
            },
        ],
        cursor: &[0],
        line_num: Some(1),
    };
    assert_eq!(vec![line_ref], rendered_lines);
}

#[test]
fn styled_length() {
    let line = Line {
        text: "some text".into(),
        cursor: vec![0],
        styles: vec![
            StyleDef {
                length: 4,
                offset: 0,
                style_id: 1,
            },
            StyleDef {
                length: 4,
                offset: 1,
                style_id: 3,
            },
        ],
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
        ranges: vec![],
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

    let mut view = View::new(From::from(1));
    view.viewport.resize(10, 10);
    view.viewport.horizontal_offset = 2;
    view.update(update);
    let rendered_lines: Vec<LineRef<'_>> = view.render_lines().collect();
    let line_ref = LineRef {
        text: "me text",
        styles: vec![
            StyleDef {
                length: 2,
                offset: 0,
                style_id: 1,
            },
            StyleDef {
                length: 4,
                offset: 1,
                style_id: 3,
            },
        ],
        cursor: &[0],
        line_num: Some(1),
    };
    assert_eq!(vec![line_ref], rendered_lines);
}

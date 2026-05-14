use crate::{
    LayoutElement, LayoutKind, LayoutLeafKind, LayoutStyle, LayoutViewport, LayoutViewportScaling,
    compute_layout, find_layout_node, hit_test,
};

fn node(
    id: &str,
    kind: LayoutKind,
    style: LayoutStyle,
    children: Vec<LayoutElement<String>>,
) -> LayoutElement<String> {
    LayoutElement {
        id: Some(id.to_owned()),
        kind,
        style,
        data: id.to_owned(),
        children,
    }
}

fn leaf(id: &str, kind: LayoutLeafKind, style: LayoutStyle) -> LayoutElement<String> {
    node(id, LayoutKind::Leaf(kind), style, Vec::new())
}

#[test]
fn column_layout_stacks_children_with_padding_and_gap() {
    let root = node(
        "root",
        LayoutKind::Column,
        LayoutStyle {
            width: Some(300.0),
            height: Some(200.0),
            padding: 10.0,
            gap: 5.0,
            ..LayoutStyle::default()
        },
        vec![
            leaf(
                "a",
                LayoutLeafKind::Spacer,
                LayoutStyle {
                    width: Some(100.0),
                    height: Some(20.0),
                    ..LayoutStyle::default()
                },
            ),
            leaf(
                "b",
                LayoutLeafKind::Spacer,
                LayoutStyle {
                    width: Some(100.0),
                    height: Some(30.0),
                    ..LayoutStyle::default()
                },
            ),
        ],
    );

    let layout = compute_layout("doc", LayoutViewport::new(300.0, 200.0), &root, None);
    let a = find_layout_node(&layout, "doc.root.a").expect("a node");
    let b = find_layout_node(&layout, "doc.root.b").expect("b node");

    assert_eq!(a.rect.x, 10.0);
    assert_eq!(a.rect.y, 10.0);
    assert_eq!(a.rect.width, 100.0);
    assert_eq!(a.rect.height, 20.0);

    assert_eq!(b.rect.x, 10.0);
    assert_eq!(b.rect.y, 35.0);
    assert_eq!(b.rect.width, 100.0);
    assert_eq!(b.rect.height, 30.0);
}

#[test]
fn row_layout_places_children_left_to_right() {
    let root = node(
        "root",
        LayoutKind::Row,
        LayoutStyle {
            width: Some(300.0),
            height: Some(80.0),
            padding: 8.0,
            gap: 4.0,
            ..LayoutStyle::default()
        },
        vec![
            leaf(
                "left",
                LayoutLeafKind::Spacer,
                LayoutStyle {
                    width: Some(50.0),
                    height: Some(20.0),
                    ..LayoutStyle::default()
                },
            ),
            leaf(
                "right",
                LayoutLeafKind::Spacer,
                LayoutStyle {
                    width: Some(70.0),
                    height: Some(20.0),
                    ..LayoutStyle::default()
                },
            ),
        ],
    );

    let layout = compute_layout("doc", LayoutViewport::new(300.0, 80.0), &root, None);
    let left = find_layout_node(&layout, "doc.root.left").expect("left node");
    let right = find_layout_node(&layout, "doc.root.right").expect("right node");

    assert_eq!(left.rect.x, 8.0);
    assert_eq!(left.rect.y, 8.0);
    assert_eq!(right.rect.x, 62.0);
    assert_eq!(right.rect.y, 8.0);
}

#[test]
fn hit_test_returns_deepest_node() {
    let root = node(
        "root",
        LayoutKind::Stack,
        LayoutStyle {
            width: Some(300.0),
            height: Some(200.0),
            ..LayoutStyle::default()
        },
        vec![leaf(
            "button",
            LayoutLeafKind::Button {
                text: "Click".to_owned(),
            },
            LayoutStyle {
                left: Some(20.0),
                top: Some(30.0),
                width: Some(100.0),
                height: Some(40.0),
                ..LayoutStyle::default()
            },
        )],
    );

    let layout = compute_layout("doc", LayoutViewport::new(300.0, 200.0), &root, None);

    assert_eq!(
        hit_test(&layout, 25.0, 35.0),
        Some("doc.root.button".to_owned())
    );
    assert_eq!(hit_test(&layout, 500.0, 500.0), None);
}

#[test]
fn fit_viewport_scales_and_centers_document() {
    let root = node(
        "root",
        LayoutKind::Stack,
        LayoutStyle {
            width: Some(100.0),
            height: Some(100.0),
            ..LayoutStyle::default()
        },
        Vec::new(),
    );

    let layout = compute_layout(
        "doc",
        LayoutViewport::new(200.0, 400.0),
        &root,
        Some((
            LayoutViewport::new(100.0, 100.0),
            LayoutViewportScaling::Fit,
        )),
    );

    assert_eq!(layout.rect.x, 0.0);
    assert_eq!(layout.rect.y, 100.0);
    assert_eq!(layout.rect.width, 200.0);
    assert_eq!(layout.rect.height, 200.0);
}

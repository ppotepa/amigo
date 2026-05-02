#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct BrowserRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct StartDialogLayout {
    pub header: BrowserRect,
    pub body: BrowserRect,
    pub mods_panel: BrowserRect,
    pub selected_panel: BrowserRect,
    pub footer: BrowserRect,
}

impl StartDialogLayout {
    pub fn fixed(width: f32, height: f32) -> Self {
        let header_h = 72.0;
        let footer_h = 52.0;
        let gap = 16.0;
        let body_h = (height - header_h - footer_h - gap * 2.0).max(240.0);
        let mods_w = 420.0;
        let selected_w = (width - mods_w - gap).max(360.0);

        Self {
            header: BrowserRect {
                x: 0.0,
                y: 0.0,
                width,
                height: header_h,
            },
            body: BrowserRect {
                x: 0.0,
                y: header_h + gap,
                width,
                height: body_h,
            },
            mods_panel: BrowserRect {
                x: 0.0,
                y: header_h + gap,
                width: mods_w,
                height: body_h,
            },
            selected_panel: BrowserRect {
                x: mods_w + gap,
                y: header_h + gap,
                width: selected_w,
                height: body_h,
            },
            footer: BrowserRect {
                x: 0.0,
                y: header_h + gap + body_h + gap,
                width,
                height: footer_h,
            },
        }
    }
}

#[cfg(feature = "taffy-layout")]
pub fn compute_taffy_start_dialog_layout(width: f32, height: f32) -> Result<StartDialogLayout, String> {
    use taffy::prelude::*;

    let mut tree: TaffyTree<()> = TaffyTree::new();
    let header = tree
        .new_leaf(Style {
            size: Size {
                width: percent(1.0),
                height: length(72.0),
            },
            ..Default::default()
        })
        .map_err(|err| format!("{err:?}"))?;
    let mods_panel = tree
        .new_leaf(Style {
            size: Size {
                width: length(340.0),
                height: percent(1.0),
            },
            ..Default::default()
        })
        .map_err(|err| format!("{err:?}"))?;
    let selected_panel = tree
        .new_leaf(Style {
            flex_grow: 1.0,
            size: Size {
                width: auto(),
                height: percent(1.0),
            },
            ..Default::default()
        })
        .map_err(|err| format!("{err:?}"))?;
    let body = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                gap: Size {
                    width: length(16.0),
                    height: length(0.0),
                },
                flex_grow: 1.0,
                size: Size {
                    width: percent(1.0),
                    height: auto(),
                },
                ..Default::default()
            },
            &[mods_panel, selected_panel],
        )
        .map_err(|err| format!("{err:?}"))?;
    let footer = tree
        .new_leaf(Style {
            size: Size {
                width: percent(1.0),
                height: length(52.0),
            },
            ..Default::default()
        })
        .map_err(|err| format!("{err:?}"))?;
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                gap: Size {
                    width: length(0.0),
                    height: length(16.0),
                },
                size: Size {
                    width: length(width),
                    height: length(height),
                },
                ..Default::default()
            },
            &[header, body, footer],
        )
        .map_err(|err| format!("{err:?}"))?;

    tree.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        },
    )
    .map_err(|err| format!("{err:?}"))?;

    let root_layout = tree.layout(root).map_err(|err| format!("{err:?}"))?;
    let header_layout = tree.layout(header).map_err(|err| format!("{err:?}"))?;
    let body_layout = tree.layout(body).map_err(|err| format!("{err:?}"))?;
    let mods_layout = tree.layout(mods_panel).map_err(|err| format!("{err:?}"))?;
    let selected_layout = tree.layout(selected_panel).map_err(|err| format!("{err:?}"))?;
    let footer_layout = tree.layout(footer).map_err(|err| format!("{err:?}"))?;

    let root_x = root_layout.location.x;
    let root_y = root_layout.location.y;
    let body_x = root_x + body_layout.location.x;
    let body_y = root_y + body_layout.location.y;

    Ok(StartDialogLayout {
        header: rect_from_layout(root_x, root_y, header_layout),
        body: rect_from_layout(root_x, root_y, body_layout),
        mods_panel: rect_from_layout(body_x, body_y, mods_layout),
        selected_panel: rect_from_layout(body_x, body_y, selected_layout),
        footer: rect_from_layout(root_x, root_y, footer_layout),
    })
}

#[cfg(feature = "taffy-layout")]
fn rect_from_layout(offset_x: f32, offset_y: f32, layout: &taffy::prelude::Layout) -> BrowserRect {
    BrowserRect {
        x: offset_x + layout.location.x,
        y: offset_y + layout.location.y,
        width: layout.size.width,
        height: layout.size.height,
    }
}

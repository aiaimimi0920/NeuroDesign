//! 固定主题预览卡片；缩略图由已准备的只读绘制对象提供，不执行资源加载。
use super::*;
unsafe fn fill(dc: HDC, rect: RECT, rgb: u32) {
    SetDCBrushColor(dc, COLORREF(rgb));
    FillRect(dc, &rect, HBRUSH(GetStockObject(DC_BRUSH).0));
}
unsafe fn text(dc: HDC, text: &str, mut rect: RECT, color: u32, flags: DRAW_TEXT_FORMAT) {
    SetTextColor(dc, COLORREF(color));
    let mut text: Vec<_> = text.encode_utf16().collect();
    DrawTextW(
        dc,
        &mut text,
        &mut rect,
        DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS | flags,
    );
}
pub(super) unsafe fn shell(dc: HDC, data: &Data) {
    let l = data.layout.borrow().clone();
    let rect = RECT {
        right: l.width,
        bottom: l.height,
        ..Default::default()
    };
    fill(dc, rect, 0x120d0a);
    SetDCBrushColor(dc, COLORREF(0x443a33));
    FrameRect(dc, &rect, HBRUSH(GetStockObject(DC_BRUSH).0));
    let old = SelectObject(dc, HGDIOBJ(data.font.get().0));
    SetBkMode(dc, TRANSPARENT);
    let caption = RECT {
        left: px(22, l.dpi),
        top: px(6, l.dpi) - l.offset.round() as i32,
        right: l.width - px(22, l.dpi),
        bottom: px(20, l.dpi) - l.offset.round() as i32,
    };
    text(dc, &data.title.borrow(), caption, 0x9f9a92, DT_LEFT);
    text(dc, &data.status.borrow(), caption, 0x9f9a92, DT_RIGHT);
    if let Some(track) = l.track() {
        fill(dc, frame(track), 0x443a33);
    }
    if let Some(thumb) = l.thumb() {
        fill(dc, frame(thumb), 0x9f9a92);
    }
    SelectObject(dc, old);
}
fn frame(f: Frame) -> RECT {
    RECT {
        left: f.x,
        top: f.y,
        right: f.x + f.width,
        bottom: f.y + f.height,
    }
}
pub(super) unsafe fn card(data: &Data, item: &DRAWITEMSTRUCT) {
    let Some(index) = data.buttons.iter().position(|h| *h == item.hwndItem) else {
        return;
    };
    let card = data.cards.borrow()[index].clone();
    let dpi = data.layout.borrow().dpi;
    let p = |n| px(n, dpi);
    let old = SaveDC(item.hDC);
    fill(
        item.hDC,
        item.rcItem,
        if item.itemState.0 & ODS_SELECTED.0 != 0 {
            0x2c2218
        } else {
            0x201711
        },
    );
    let thumb = RECT {
        left: p(7),
        top: p(7),
        right: item.rcItem.right - p(7),
        bottom: p(61),
    };
    fill(item.hDC, thumb, 0x110c09);
    if let Some(preview) = &card.preview {
        preview.paint(item.hDC, thumb);
    }
    SelectObject(item.hDC, HGDIOBJ(data.font.get().0));
    SetBkMode(item.hDC, TRANSPARENT);
    text(
        item.hDC,
        &card.title,
        RECT {
            top: p(65),
            bottom: p(79),
            ..thumb
        },
        if card.enabled { 0xeff8f7 } else { 0x9f9a92 },
        DT_LEFT,
    );
    text(
        item.hDC,
        &card.detail,
        RECT {
            top: p(82),
            bottom: p(96),
            ..thumb
        },
        0x9f9a92,
        DT_LEFT,
    );
    SetDCBrushColor(
        item.hDC,
        COLORREF(
            if data.selected.get() == Some(index) || item.itemState.0 & ODS_FOCUS.0 != 0 {
                0x38ffd9
            } else {
                0x443a33
            },
        ),
    );
    FrameRect(item.hDC, &item.rcItem, HBRUSH(GetStockObject(DC_BRUSH).0));
    let _ = RestoreDC(item.hDC, old);
}

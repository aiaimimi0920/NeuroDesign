//! 悬浮条的 DIP 几何；控件状态变化不改变槽位，窄工作区只换行，不缩小图标。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Layout {
    pub width: i32,
    pub height: i32,
    pub slots: Vec<Frame>,
    pub dividers: Vec<Frame>,
}
pub(crate) fn px(value: i32, dpi: u32) -> i32 {
    ((i64::from(value) * i64::from(dpi.max(96)) + 48) / 96) as i32
}
impl Layout {
    pub fn new(items: &[super::Item], dpi: u32, max_width: i32) -> Self {
        let desired = 12
            + items
                .iter()
                .map(|item| match item {
                    super::Item::Divider => 8,
                    super::Item::Button(_) => 28,
                    super::Item::Slot(_) => 0,
                })
                .sum::<i32>();
        let width = px(desired, dpi).min(max_width.max(px(40, dpi)));
        let (mut x, mut y) = (6, 5);
        let mut slots = Vec::new();
        let mut dividers = Vec::new();
        for item in items {
            if matches!(item, super::Item::Slot(_)) {
                continue;
            }
            let divider = matches!(item, super::Item::Divider);
            let w = if divider { 8 } else { 28 };
            if divider && px(x + w + 6, dpi) > width {
                continue;
            }
            // 在 DIP 中累加，最后投影各条边；125% 等 DPI 不因逐项舍入多出一行。
            if px(x + w + 6, dpi) > width && x > 6 {
                x = 6;
                y += 28;
            }
            let frame = Frame {
                x: px(x, dpi),
                y: px(y, dpi),
                width: px(x + w, dpi) - px(x, dpi),
                height: px(28, dpi),
            };
            if divider {
                dividers.push(frame)
            } else {
                slots.push(frame)
            }
            x += w;
        }
        Self {
            width,
            height: px(y + 33, dpi),
            slots,
            dividers,
        }
    }
}

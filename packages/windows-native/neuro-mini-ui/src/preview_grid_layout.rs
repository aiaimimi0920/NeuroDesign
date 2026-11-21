//! 右附着预览格的纯几何；选择状态不改变尺寸，滚动不影响相邻参数 rail。
use crate::{layout::px, Frame};
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewGridLayout {
    pub width: i32,
    pub height: i32,
    pub content: i32,
    pub cards: Vec<Frame>,
    pub offset: f64,
    pub dpi: u32,
}
impl PreviewGridLayout {
    pub fn new(count: usize, dpi: u32, offset: f64) -> Self {
        let dpi = dpi.clamp(96, 768);
        let count = count.min(64);
        let columns = count.clamp(1, 2);
        let width = px(250, dpi);
        let inner = width - px(44, dpi);
        let gap = px(8, dpi);
        let card = (inner - gap * (columns - 1) as i32) / columns as i32;
        let content = px(26, dpi) + count.div_ceil(columns) as i32 * px(108, dpi) + px(6, dpi);
        let height = content.min(px(320, dpi));
        let maximum = (content - height).max(0) as f64;
        let offset = if offset.is_finite() {
            offset.clamp(0.0, maximum)
        } else {
            0.0
        };
        let cards = (0..count)
            .map(|i| Frame {
                x: px(22, dpi) + i as i32 % columns as i32 * (card + gap),
                y: px(26, dpi) + i as i32 / columns as i32 * px(108, dpi) - offset.round() as i32,
                width: card,
                height: px(100, dpi),
            })
            .collect();
        Self {
            width,
            height,
            content,
            cards,
            offset,
            dpi,
        }
    }
    pub fn track(&self) -> Option<Frame> {
        (self.content > self.height).then(|| Frame {
            x: self.width - px(6, self.dpi),
            y: px(6, self.dpi),
            width: px(4, self.dpi),
            height: self.height - px(12, self.dpi),
        })
    }
    pub fn thumb(&self) -> Option<Frame> {
        let track = self.track()?;
        let height = (track.height * self.height / self.content)
            .max(px(18, self.dpi))
            .min(track.height);
        Some(Frame {
            y: track.y
                + (self.offset / (self.content - self.height) as f64
                    * (track.height - height) as f64)
                    .round() as i32,
            height,
            ..track
        })
    }
}

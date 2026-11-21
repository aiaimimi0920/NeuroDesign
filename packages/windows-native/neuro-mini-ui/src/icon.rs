//! 在 BeginDraw 前准备完整图标，所有元素共用调用方的 currentColor 与禁用透明度。
use crate::{
    icon_svg::{self, Shape, Style},
    paint::{color, Painter},
    Icon,
};
use windows::{core::Result, Win32::Graphics::Direct2D::*};
struct Part {
    geometry: ID2D1PathGeometry,
    style: Style,
    stroke: ID2D1StrokeStyle,
}
pub(crate) struct Prepared {
    parts: Vec<Part>,
    scale: f32,
}
pub(crate) unsafe fn prepare(
    p: &Painter,
    icon: Icon,
    scale: f32,
    offset: (f32, f32),
) -> Result<Prepared> {
    let (factor, shapes) = if icon.svg {
        icon_svg::parse(icon.source)?
    } else {
        (
            1.0,
            vec![Shape {
                path: icon.source.into(),
                style: Style {
                    fill: None,
                    stroke: Some(1.0),
                    fill_color: None,
                    stroke_color: None,
                    opacity: 1.0,
                    width: 1.6,
                    round_cap: false,
                    round_join: false,
                    dash: None,
                },
                rotation: (0.0, 0.0, 0.0),
            }],
        )
    };
    let scale = scale * factor;
    let mut parts = Vec::with_capacity(shapes.len());
    for shape in shapes {
        let style = shape.style;
        let cap = if style.round_cap {
            D2D1_CAP_STYLE_ROUND
        } else {
            D2D1_CAP_STYLE_FLAT
        };
        let dashes = style.dash.map(|(a, b)| [a / style.width, b / style.width]);
        parts.push(Part {
            geometry: crate::path::geometry_rotated(
                &p.factory,
                &shape.path,
                scale,
                offset,
                style.fill.is_some(),
                shape.rotation,
            )?,
            stroke: p.factory.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES {
                    startCap: cap,
                    endCap: cap,
                    dashCap: cap,
                    dashStyle: if dashes.is_some() {
                        D2D1_DASH_STYLE_CUSTOM
                    } else {
                        D2D1_DASH_STYLE_SOLID
                    },
                    lineJoin: if style.round_join {
                        D2D1_LINE_JOIN_ROUND
                    } else {
                        D2D1_LINE_JOIN_MITER
                    },
                    miterLimit: 4.0,
                    ..Default::default()
                },
                dashes.as_ref().map(|v| v.as_slice()),
            )?,
            style,
        });
    }
    Ok(Prepared { parts, scale })
}
impl Prepared {
    pub(crate) unsafe fn draw(
        &self,
        p: &Painter,
        brush: &ID2D1SolidColorBrush,
        ink: u32,
        opacity: f32,
    ) {
        for part in &self.parts {
            let style = part.style;
            if let Some(alpha) = style.fill {
                brush.SetColor(&color(
                    style.fill_color.unwrap_or(ink),
                    alpha * style.opacity * opacity,
                ));
                p.target.FillGeometry(&part.geometry, brush, None);
            }
            if let Some(alpha) = style.stroke {
                brush.SetColor(&color(
                    style.stroke_color.unwrap_or(ink),
                    alpha * style.opacity * opacity,
                ));
                p.target.DrawGeometry(
                    &part.geometry,
                    brush,
                    style.width * self.scale,
                    &part.stroke,
                );
            }
        }
    }
}

//! 静态图标的有限 SVG 子集。保留原元素顺序及 fill/stroke，不接受外部内容。
use roxmltree::Node;
use windows::{
    core::{Error, Result},
    Win32::Foundation::E_INVALIDARG,
};
#[derive(Clone, Copy)]
pub(crate) struct Style {
    pub fill: Option<f32>,
    pub stroke: Option<f32>,
    pub fill_color: Option<u32>,
    pub stroke_color: Option<u32>,
    pub opacity: f32,
    pub width: f32,
    pub round_cap: bool,
    pub round_join: bool,
    pub dash: Option<(f32, f32)>,
}
pub(crate) struct Shape {
    pub path: String,
    pub style: Style,
    pub rotation: (f32, f32, f32),
}
fn invalid() -> Error {
    Error::new(E_INVALIDARG, "Unsupported static toolbar SVG")
}
fn number(value: &str) -> Result<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(invalid)
}
fn numeric(node: Node<'_, '_>, key: &str, default: f32) -> Result<f32> {
    node.attribute(key).map_or(Ok(default), number)
}
fn paint(value: &str) -> Result<(Option<f32>, Option<u32>)> {
    match value {
        "none" => Ok((None, None)),
        "currentColor" => Ok((Some(1.0), None)),
        _ if value.len() == 7 && value.starts_with('#') => {
            let hex = &value[1..];
            if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(invalid());
            }
            Ok((
                Some(1.0),
                Some(u32::from_str_radix(hex, 16).map_err(|_| invalid())?),
            ))
        }
        _ => Err(invalid()),
    }
}
fn style(node: Node<'_, '_>, inherited: Style) -> Result<Style> {
    let mut s = inherited;
    for (key, target, color) in [
        ("fill", &mut s.fill, &mut s.fill_color),
        ("stroke", &mut s.stroke, &mut s.stroke_color),
    ] {
        if let Some(value) = node.attribute(key) {
            (*target, *color) = paint(value)?;
        }
    }
    s.fill = s
        .fill
        .map(|a| numeric(node, "fill-opacity", a))
        .transpose()?;
    s.stroke = s
        .stroke
        .map(|a| numeric(node, "stroke-opacity", a))
        .transpose()?;
    s.opacity *= numeric(node, "opacity", 1.0)?;
    s.width = numeric(node, "stroke-width", s.width)?;
    if let Some(raw) = node.attribute("stroke-dasharray") {
        let list = raw
            .split_whitespace()
            .map(number)
            .collect::<Result<Vec<_>>>()?;
        if list.len() != 2 || list.iter().any(|v| *v <= 0.0) || s.width <= 0.0 {
            return Err(invalid());
        }
        s.dash = Some((list[0], list[1]));
    }
    if let Some(value) = node.attribute("stroke-linecap") {
        s.round_cap = match value {
            "round" => true,
            "butt" => false,
            _ => return Err(invalid()),
        };
    }
    if let Some(value) = node.attribute("stroke-linejoin") {
        s.round_join = match value {
            "round" => true,
            "miter" => false,
            _ => return Err(invalid()),
        };
    }
    if s.width < 0.0
        || !(0.0..=1.0).contains(&s.opacity)
        || s.fill
            .into_iter()
            .chain(s.stroke)
            .any(|a| !(0.0..=1.0).contains(&a))
    {
        return Err(invalid());
    }
    Ok(s)
}
fn path(node: Node<'_, '_>) -> Result<String> {
    let n = |key, default| numeric(node, key, default);
    Ok(match node.tag_name().name() {
        "path" => node.attribute("d").ok_or_else(invalid)?.into(),
        "rect" => {
            let (x, y, w, h) = (
                n("x", 0.0)?,
                n("y", 0.0)?,
                n("width", 0.0)?,
                n("height", 0.0)?,
            );
            if w <= 0.0 || h <= 0.0 {
                return Err(invalid());
            }
            let rx = n("rx", n("ry", 0.0)?)?.clamp(0.0, w / 2.0);
            let ry = n("ry", rx)?.clamp(0.0, h / 2.0);
            let (r, b) = (x + w, y + h);
            if rx == 0.0 || ry == 0.0 {
                format!("M{x} {y}H{r}V{b}H{x}Z")
            } else {
                format!("M{} {y}H{}A{rx} {ry} 0 0 1 {r} {}V{}A{rx} {ry} 0 0 1 {} {b}H{}A{rx} {ry} 0 0 1 {x} {}V{}A{rx} {ry} 0 0 1 {} {y}Z",x+rx,r-rx,y+ry,b-ry,r-rx,x+rx,b-ry,y+ry,x+rx)
            }
        }
        "circle" | "ellipse" => {
            let (x, y) = (n("cx", 0.0)?, n("cy", 0.0)?);
            let rx = n("rx", n("r", 0.0)?)?;
            let ry = n("ry", rx)?;
            if rx <= 0.0 || ry <= 0.0 {
                return Err(invalid());
            }
            format!(
                "M{} {y}a{rx} {ry} 0 1 0 {} 0a{rx} {ry} 0 1 0 {} 0Z",
                x - rx,
                2.0 * rx,
                -2.0 * rx
            )
        }
        _ => return Err(invalid()),
    })
}
pub(crate) fn parse(source: &str) -> Result<(f32, Vec<Shape>)> {
    if source.len() > 65536 {
        return Err(invalid());
    }
    let doc = roxmltree::Document::parse(source).map_err(|_| invalid())?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err(invalid());
    }
    let view: Vec<_> = root
        .attribute("viewBox")
        .ok_or_else(invalid)?
        .split_whitespace()
        .map(number)
        .collect::<Result<_>>()?;
    if view.len() != 4 || view[0] != 0.0 || view[1] != 0.0 || view[2] <= 0.0 || view[2] != view[3] {
        return Err(invalid());
    }
    let base = style(
        root,
        Style {
            fill: Some(1.0),
            stroke: None,
            fill_color: None,
            stroke_color: None,
            opacity: 1.0,
            width: 1.0,
            round_cap: false,
            round_join: false,
            dash: None,
        },
    )?;
    let mut shapes = Vec::new();
    for node in root.descendants().filter(|n| n.is_element()) {
        if node.attributes().any(|a| {
            !matches!(
                a.name(),
                "viewBox"
                    | "xmlns"
                    | "fill"
                    | "stroke"
                    | "opacity"
                    | "fill-opacity"
                    | "stroke-opacity"
                    | "stroke-width"
                    | "stroke-linecap"
                    | "stroke-linejoin"
                    | "stroke-dasharray"
                    | "d"
                    | "x"
                    | "y"
                    | "width"
                    | "height"
                    | "rx"
                    | "ry"
                    | "cx"
                    | "cy"
                    | "r"
                    | "transform"
            )
        }) {
            return Err(invalid());
        }
        if node == root {
            continue;
        }
        if node.parent_element() != Some(root) || shapes.len() >= 64 {
            return Err(invalid());
        }
        let rotation = if let Some(raw) = node.attribute("transform") {
            let raw = raw
                .strip_prefix("rotate(")
                .and_then(|s| s.strip_suffix(')'))
                .ok_or_else(invalid)?;
            let values = raw
                .split_whitespace()
                .map(number)
                .collect::<Result<Vec<_>>>()?;
            if values.len() != 3 {
                return Err(invalid());
            }
            (values[0], values[1], values[2])
        } else {
            (0.0, 0.0, 0.0)
        };
        shapes.push(Shape {
            path: path(node)?,
            style: style(node, base)?,
            rotation,
        });
    }
    if shapes.is_empty() {
        return Err(invalid());
    }
    Ok((24.0 / view[2], shapes))
}

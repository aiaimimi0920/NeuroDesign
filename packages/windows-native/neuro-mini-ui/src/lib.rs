//! Neuro-mini 的原生悬浮工具条；只拥有表现、控件与短暂输入状态，不执行应用命令。
#![cfg(windows)]
mod button;
pub use button::install as prepare_button;
mod button_hold;
mod capsule;
mod capsule_layout;
pub use capsule::{CapsuleAction, CapsuleAppearance, CapsulePainter};
pub use capsule_layout::{CapsuleLayout, CapsuleSide, CapsuleSize};
mod color;
mod drawer;
mod drawer_layout;
pub use color::{track_color, ColorAction, ColorIcons, ColorOptions, ColorResult, Rgba};
pub use drawer::{Drawer, DrawerButton, DRAWER_TOGGLE};
pub use drawer_layout::DrawerLayout;
mod icon;
mod icon_svg;
mod layout;
mod list;
pub use list::{track_list, track_options, ListOption};
mod menu;
mod notice;
pub use notice::{Notice, NoticeStack};
mod paint;
pub mod paint_batch;
mod path;
mod preview_grid;
mod preview_grid_layout;
mod properties;
pub use preview_grid::{CardPreview, PreviewCard, PreviewGrid};
pub use preview_grid_layout::PreviewGridLayout;
mod rail_layout;
mod strip_shell;
pub use properties::{
    attached_position, property_anchor, property_contains, property_window, set_properties,
    set_property_expanded, sync_attached, take_property_edit, translate_properties, PropertyEdit,
    PropertyField, PropertyKind, PropertyPanel, PANEL, PANEL_CLOSE, PROPERTY,
};
pub use rail_layout::RailLayout;
mod toolbar;
pub use layout::{Frame, Layout};
pub use menu::{track_menu, MenuItem};
pub use paint::{Icon, Palette};
pub use toolbar::{
    create, is_toolbar, owner, prepare, ready, resize, set_text_slots, update, Button, Item,
    TextSlot, CLOSE, PREPARED,
};

#[cfg(test)]
mod button_hold_tests;
#[cfg(test)]
mod icon_tests;
#[cfg(test)]
#[path = "toolbar_present_tests.rs"]
mod present_tests;
#[cfg(test)]
mod tests;

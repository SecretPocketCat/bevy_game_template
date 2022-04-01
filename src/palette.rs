use bevy::prelude::*;
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{DrawMode, FillMode, GeometryBuilder, StrokeMode},
};

pub struct PalettePlugin;
impl Plugin for PalettePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_sprite_added)
            .add_system(on_text_added)
            .add_system(on_shape_palette_color_changed)
            .add_system(on_ui_color_added)
            .insert_resource(Palette {
                background: Color::hsl(0., 0., 0.23),
                text: Color::hsl(0., 0., 1.),
                button: Color::hsl(0., 0., 0.45),
                button_focus: Color::hsl(0., 0., 0.60),
                button_active: Color::hsl(0., 0., 0.80),
                button_accent: Color::rgb_u8(0, 68, 115),
                button_accent_focus: Color::rgb_u8(0, 60, 150),
                button_accent_active: Color::rgb_u8(54, 0, 162),
                button_text: Color::hsl(0., 0., 0.75),
                button_text_focus: Color::hsl(0., 0., 0.85),
                button_text_active: Color::hsl(0., 0., 0.95),
                button_text_accent: Color::hsl(0., 0., 0.75),
                button_text_accent_focus: Color::hsl(0., 0., 0.85),
                button_text_accent_active: Color::hsl(0., 0., 0.95),
            });
    }
}

pub struct Palette {
    background: Color,
    text: Color,
    button: Color,
    button_focus: Color,
    button_active: Color,
    button_accent: Color,
    button_accent_focus: Color,
    button_accent_active: Color,
    button_text: Color,
    button_text_focus: Color,
    button_text_active: Color,
    button_text_accent: Color,
    button_text_accent_focus: Color,
    button_text_accent_active: Color,
}

impl Palette {
    pub fn get_color(&self, col: &PaletteColor) -> Color {
        match col {
            PaletteColor::Background => self.background,
            PaletteColor::Text => self.text,
            PaletteColor::Button => self.button,
            PaletteColor::ButtonFocus => self.button_focus,
            PaletteColor::ButtonActive => self.button_active,
            PaletteColor::ButtonAccent => self.button_accent,
            PaletteColor::ButtonAccentFocus => self.button_accent_focus,
            PaletteColor::ButtonAccentActive => self.button_accent_active,
            PaletteColor::ButtonText => self.button_text,
            PaletteColor::ButtonTextFocus => self.button_text_focus,
            PaletteColor::ButtonTextActive => self.button_text_active,
            PaletteColor::ButtonTextAccent => self.button_text_accent,
            PaletteColor::ButtonTextAccentFocus => self.button_text_accent_focus,
            PaletteColor::ButtonTextAccentActive => self.button_text_accent_active,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum PaletteColor {
    Background,
    Text,
    Button,
    ButtonFocus,
    ButtonActive,
    ButtonAccent,
    ButtonAccentFocus,
    ButtonAccentActive,
    ButtonText,
    ButtonTextFocus,
    ButtonTextActive,
    ButtonTextAccent,
    ButtonTextAccentFocus,
    ButtonTextAccentActive,
}

#[derive(Component, Clone, Copy)]
pub enum ShapePaletteColor {
    Fill(PaletteColor),
    Stroke(Stroke),
    Outlined {
        fill_color: PaletteColor,
        stroke: Stroke,
    },
}

#[derive(Clone, Copy)]
pub struct Stroke {
    pub color: PaletteColor,
    pub width: f32,
}

fn on_sprite_added(
    palette: Res<Palette>,
    mut q: Query<(&PaletteColor, &mut Sprite), Added<Sprite>>,
) {
    for (col, mut sprite) in q.iter_mut() {
        sprite.color = palette.get_color(col);
    }
}

fn on_text_added(palette: Res<Palette>, mut q: Query<(&PaletteColor, &mut Text), Added<Text>>) {
    for (col, mut text) in q.iter_mut() {
        text.sections[0].style.color = palette.get_color(col);
    }
}

fn on_ui_color_added(
    palette: Res<Palette>,
    mut q: Query<(&PaletteColor, &mut UiColor), Added<Text>>,
) {
    for (col, mut ui_col) in q.iter_mut() {
        ui_col.0 = palette.get_color(col);
    }
}

fn on_shape_palette_color_changed(
    palette: Res<Palette>,
    mut q: Query<(&mut DrawMode, &ShapePaletteColor), Changed<ShapePaletteColor>>,
) {
    for (mut draw_mode, shape_col) in q.iter_mut() {
        match shape_col {
            ShapePaletteColor::Fill(col) => {
                *draw_mode = DrawMode::Fill(FillMode::color(palette.get_color(col)));
            }
            ShapePaletteColor::Stroke(stroke) => {
                *draw_mode = DrawMode::Stroke(StrokeMode::new(
                    palette.get_color(&stroke.color),
                    stroke.width,
                ));
            }
            ShapePaletteColor::Outlined {
                fill_color: fill,
                stroke,
            } => {
                *draw_mode = DrawMode::Outlined {
                    fill_mode: FillMode::color(palette.get_color(fill)),
                    outline_mode: StrokeMode::new(palette.get_color(&stroke.color), stroke.width),
                };
            }
        }
    }
}

pub fn get_shape_bundle_from_xyz(x: f32, y: f32, z: f32) -> ShapeBundle {
    GeometryBuilder::default().build(
        DrawMode::Fill(FillMode::color(Color::GRAY)),
        Transform::from_xyz(x, y, z),
    )
}

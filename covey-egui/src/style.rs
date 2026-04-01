use std::{collections::BTreeMap, sync::Arc};

use anyhow::Context;
use egui::{
    Color32, CornerRadius, Margin, Shadow, Spacing, Stroke, Vec2, Visuals,
    style::{
        Interaction, ScrollAnimation, ScrollStyle, Selection, TextCursorStyle, WidgetVisuals,
        Widgets,
    },
};
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};

pub fn style_reset() -> egui::Style {
    let empty_visuals = WidgetVisuals {
        bg_fill: Color32::TRANSPARENT,
        weak_bg_fill: Color32::TRANSPARENT,
        bg_stroke: Stroke::NONE,
        corner_radius: CornerRadius::ZERO,
        fg_stroke: Stroke::NONE,
        expansion: 0.0,
    };

    egui::Style {
        override_text_style: None,
        override_font_id: None,
        override_text_valign: None,
        text_styles: BTreeMap::new(),
        // drag_value_text_style: todo!(),
        // number_formatter: todo!(),
        // wrap: todo!(),
        wrap_mode: None,
        spacing: Spacing {
            item_spacing: Vec2::ZERO,
            window_margin: Margin::ZERO,
            button_padding: Vec2::ZERO,
            menu_margin: Margin::ZERO,
            indent: 0.0,
            // interact_size: todo!(),
            slider_width: 0.0,
            slider_rail_height: 0.0,
            combo_width: 0.0,
            text_edit_width: 0.0,
            icon_width: 0.0,
            icon_width_inner: 0.0,
            icon_spacing: 0.0,
            // default_area_size: todo!(),
            tooltip_width: 0.0,
            menu_width: 0.0,
            menu_spacing: 0.0,
            // indent_ends_with_horizontal_line: todo!(),
            combo_height: 0.0,
            scroll: ScrollStyle::floating(),
            ..Spacing::default()
        },
        interaction: Interaction::default(),
        visuals: Visuals {
            dark_mode: true,
            // text_alpha_from_coverage: (),
            override_text_color: None,
            weak_text_alpha: 1.0,
            weak_text_color: None,
            widgets: Widgets {
                noninteractive: empty_visuals,
                inactive: empty_visuals,
                hovered: empty_visuals,
                active: empty_visuals,
                open: empty_visuals,
            },
            selection: Selection {
                bg_fill: Color32::TRANSPARENT,
                stroke: Stroke::NONE,
            },
            hyperlink_color: Color32::TRANSPARENT,
            faint_bg_color: Color32::TRANSPARENT,
            extreme_bg_color: Color32::TRANSPARENT,
            text_edit_bg_color: None,
            code_bg_color: Color32::TRANSPARENT,
            warn_fg_color: Color32::TRANSPARENT,
            error_fg_color: Color32::TRANSPARENT,
            window_corner_radius: CornerRadius::ZERO,
            window_shadow: Shadow::NONE,
            window_fill: Color32::TRANSPARENT,
            window_stroke: Stroke::NONE,
            window_highlight_topmost: true,
            menu_corner_radius: CornerRadius::ZERO,
            panel_fill: Color32::TRANSPARENT,
            popup_shadow: Shadow::NONE,
            resize_corner_size: 0.0,
            text_cursor: TextCursorStyle::default(),
            // clip_rect_margin: 0.0,
            button_frame: true,
            collapsing_header_frame: true,
            indent_has_left_vline: false,
            striped: false,
            slider_trailing_fill: false,
            // handle_shape: (),
            interact_cursor: None,
            image_loading_spinners: true,
            // numeric_color_space: (),
            disabled_alpha: 1.0,
            ..Visuals::dark()
        },
        animation_time: 0.0,
        #[cfg(debug_assertions)]
        debug: egui::style::DebugOptions::default(),
        explanation_tooltips: false,
        url_in_tooltip: false,
        always_scroll_the_only_direction: true,
        scroll_animation: ScrollAnimation::none(),
        // compact_menu_style: todo!(),
        ..egui::Style::default()
    }
}

pub fn load_system_fonts() -> egui::FontDefinitions {
    // https://github.com/emilk/egui/discussions/1344#discussioncomment-6432960

    let mut fonts = egui::FontDefinitions::default();
    insert_fonts(
        &mut fonts,
        egui::FontFamily::Proportional,
        &["Inter", "system-ui", "Noto Sans Math"],
    );
    insert_fonts(&mut fonts, egui::FontFamily::Monospace, &["ui-monospace"]);

    fonts
}

fn insert_fonts(
    fonts: &mut egui::FontDefinitions,
    egui_family: egui::FontFamily,
    font_families: &[&str],
) {
    for ff in font_families.iter().rev() {
        let font = match find_font_from_system(ff) {
            Ok(font) => font,
            Err(e) => {
                tracing::warn!("failed to load font family {ff:?} from system: {e:#}");
                continue;
            }
        };

        fonts.font_data.insert(
            ff.to_string(),
            Arc::new(egui::FontData::from_owned(
                font.copy_font_data()
                    .expect("copy_font_data never returns none")
                    .to_vec(),
            )),
        );
    }

    fonts
        .families
        .entry(egui_family)
        .or_default()
        .splice(0..0, font_families.iter().map(|s| (*s).to_owned()));
}

fn find_font_from_system(font_family: &str) -> anyhow::Result<font_kit::font::Font> {
    Ok(match font_family {
        "system-ui" => SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())?
            .load()?,
        "ui-monospace" => SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())?
            .load()?,
        _ => SystemSource::new()
            .select_family_by_name(font_family)?
            .fonts()
            .first()
            .context("no font handles found in font")?
            .load()?,
    })
}

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::atoms::{Shape, ShapeBox};
use ui_widgets::egui::{Color32, Vec2};
use ui_widgets::traits::Sizeable;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, render_ui);
    }
}

fn render_ui(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let total_w = ui.available_width();
            let total_h = ui.available_height();

            let margin = total_w * 0.02;
            let gap = total_w * 0.01;
            let col1_w = total_w * 0.24;
            let col2_w = total_w * 0.46;
            let col3_w = total_w * 0.24;

            let top_margin = margin / 2.0;
            let col_h = total_h - top_margin;

            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            ui.add_space(top_margin);

            ui.horizontal(|ui| {
                ui.add_space(margin);

                let mut s1 = ShapeBox::new(Shape::Rectangle)
                    .fill(Color32::from_rgb(70, 130, 180));
                s1.set_max_size(Vec2::new(col1_w, col_h));
                ui.add(s1);

                ui.add_space(gap);

                let mut s2 = ShapeBox::new(Shape::Rectangle)
                    .fill(Color32::from_rgb(60, 179, 113));
                s2.set_max_size(Vec2::new(col2_w, col_h));
                ui.add(s2);

                ui.add_space(gap);

                let mut s3 = ShapeBox::new(Shape::Rectangle)
                    .fill(Color32::from_rgb(186, 85, 211));
                s3.set_max_size(Vec2::new(col3_w, col_h));
                ui.add(s3);
            });
        });

    Ok(())
}

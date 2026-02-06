use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, Characteristics, EquippedGear, IdentityBar, Inventory, Points, Portrait, Skills,
    Stats, StatusBar, Traits, Wallet,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, render_ui);
    }
}

fn render_ui(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
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

                let gap_between = col_h * 0.03 / 4.0;

                let portrait_h = col_h * 0.30;
                let identity_h = col_h * 0.11;
                let status1_h = col_h * 0.16;
                let stats_h = col_h * 0.20;
                let status2_h = col_h * 0.20;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.add_sized([col1_w, portrait_h], Portrait::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, identity_h], IdentityBar::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, status1_h], StatusBar::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, stats_h], Stats::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col1_w, status2_h], StatusBar::new());
                });

                ui.add_space(gap);

                let characteristics_h = col_h * 0.14;
                let points_h = col_h * 0.05;
                let skills_h = col_h * 0.24;
                let traits_h = col_h * 0.14;
                let abilities_h = col_h * 0.40;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.add_sized([col2_w, characteristics_h], Characteristics::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, points_h], Points::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, skills_h], Skills::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, traits_h], Traits::new());
                    ui.add_space(gap_between);
                    ui.add_sized([col2_w, abilities_h], Abilities::new());
                });

                ui.add_space(gap);

                let gap_between_col3 = col_h * 0.03 / 2.0;

                let equipped_gear_h = col_h * 0.41;
                let wallet_h = col_h * 0.08;
                let inventory_h = col_h * 0.48;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.add_sized([col3_w, equipped_gear_h], EquippedGear::new());
                    ui.add_space(gap_between_col3);
                    ui.add_sized([col3_w, wallet_h], Wallet::new());
                    ui.add_space(gap_between_col3);
                    ui.add_sized([col3_w, inventory_h], Inventory::new());
                });
            });
        });

    Ok(())
}

mod apply;
mod helpers;
mod icons;
mod layout;
mod overlays;
mod params;

pub use helpers::format_effect;
pub use overlays::{render_trait_select_overlay, TraitSelectMode};
pub use params::{EditMode, PasswordPopupOpen};

use bevy::prelude::*;
use bevy_egui::{egui, EguiPrimaryContextPass};
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};

use crate::events::{
    CreateItem, ExperienceChanged, InventoryChanged, LearnAbility, LearnTrait, LevelUp,
    ResourceChanged, UpgradeEvent, WalletChanged,
};
use crate::state::AppScreen;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditMode>()
            .init_resource::<params::LearnAbilityOpen>()
            .init_resource::<params::LearnTraitOpen>()
            .init_resource::<crate::create_item::CreateItemOpen>()
            .init_resource::<params::PasswordPopupOpen>()
            .add_message::<ResourceChanged>()
            .add_message::<WalletChanged>()
            .add_message::<InventoryChanged>()
            .add_message::<ExperienceChanged>()
            .add_message::<LevelUp>()
            .add_message::<UpgradeEvent>()
            .add_message::<LearnAbility>()
            .add_message::<LearnTrait>()
            .add_message::<CreateItem>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    icons::init_icons.run_if(not(resource_exists::<icons::UiIcons>)),
                    layout::render_ui.run_if(in_state(AppScreen::CharacterSheet)),
                ),
            )
            .add_systems(
                Update,
                (
                    apply::apply_resource_changes,
                    apply::apply_wallet_changes,
                    apply::apply_inventory_changes,
                    apply::apply_experience_changes,
                    apply::apply_level_up,
                    apply::apply_upgrades,
                    apply::apply_learn_ability,
                    apply::apply_learn_trait,
                ),
            )
            .add_systems(Update, apply::apply_create_item);
    }
}

/// Renders the password input popup for authentication.
/// Used by both the character sheet and character select screens.
pub fn render_password_popup(
    ctx: &egui::Context,
    popup_open: &mut PasswordPopupOpen,
    auth_state: &mut crate::network::AuthState,
    pending_messages: &mut crate::network::PendingClientMessages,
) {
    let state_id = egui::Id::new("password_popup_text");
    let mut password: String = ctx.data(|d| d.get_temp(state_id)).unwrap_or_default();

    let mut close = false;
    let mut submit = false;

    egui::Window::new("Authenticate")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(SECONDARY_COLOR)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                .inner_margin(egui::Margin::same(20)),
        )
        .min_width(300.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Enter Password")
                        .size(18.0)
                        .color(TEXT_COLOR)
                        .strong(),
                );
                ui.add_space(12.0);
            });

            let response = ui.add_sized(
                [ui.available_width(), 28.0],
                egui::TextEdit::singleline(&mut password).password(true),
            );

            // Auto-focus the password field
            if response.gained_focus() || !response.lost_focus() {
                response.request_focus();
            }

            // Submit on Enter
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                submit = true;
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                let unlock_btn = ui.add(
                    egui::Button::new(egui::RichText::new("Unlock").size(14.0).color(TEXT_COLOR))
                        .fill(MAIN_COLOR)
                        .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                        .corner_radius(4.0),
                );
                if unlock_btn.clicked() {
                    submit = true;
                }

                ui.add_space(8.0);

                let cancel_btn = ui.add(
                    egui::Button::new(egui::RichText::new("Cancel").size(14.0).color(TEXT_COLOR))
                        .fill(SECONDARY_COLOR)
                        .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                        .corner_radius(4.0),
                );
                if cancel_btn.clicked() {
                    close = true;
                }
            });
            ui.add_space(4.0);
        });

    if submit && !password.is_empty() {
        pending_messages.push(shared::ClientMessage::Authenticate {
            password: password.clone(),
        });
        auth_state.pending = true;
        close = true;
    }

    if close {
        popup_open.0 = false;
        ctx.data_mut(|d| d.remove::<String>(state_id));
    } else {
        ctx.data_mut(|d| d.insert_temp(state_id, password));
    }
}

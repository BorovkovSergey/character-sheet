mod apply;
mod helpers;
mod icons;
mod layout;
mod overlays;
mod params;

pub use params::EditMode;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

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

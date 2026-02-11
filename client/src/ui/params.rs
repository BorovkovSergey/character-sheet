use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::events::{
    CreateItem, ExperienceChanged, InventoryChanged, LearnAbility, LearnTrait, ResourceChanged,
    UpgradeEvent, WalletChanged,
};

#[derive(Resource, Default)]
pub struct EditMode(pub bool);

#[derive(Resource, Default)]
pub(super) struct LearnAbilityOpen(pub bool);

#[derive(Resource, Default)]
pub(super) struct LearnTraitOpen(pub bool);

#[derive(SystemParam)]
pub(super) struct UiEvents<'w> {
    pub resource: MessageWriter<'w, ResourceChanged>,
    pub wallet: MessageWriter<'w, WalletChanged>,
    pub inventory: MessageWriter<'w, InventoryChanged>,
    pub experience: MessageWriter<'w, ExperienceChanged>,
    pub upgrade: MessageWriter<'w, UpgradeEvent>,
    pub learn_ability: MessageWriter<'w, LearnAbility>,
    pub learn_trait: MessageWriter<'w, LearnTrait>,
    pub create_item: MessageWriter<'w, CreateItem>,
}

#[derive(SystemParam)]
pub(super) struct Registries<'w> {
    pub traits: Res<'w, crate::network::ClientTraitRegistry>,
    pub skills: Res<'w, crate::network::ClientSkillRegistry>,
    pub abilities: Res<'w, crate::network::ClientAbilityRegistry>,
    pub weapons: Res<'w, crate::network::ClientWeaponRegistry>,
    pub equipment: Res<'w, crate::network::ClientEquipmentRegistry>,
    pub items: Res<'w, crate::network::ClientItemRegistry>,
}

#[derive(SystemParam)]
pub(super) struct UiModals<'w> {
    pub edit_mode: ResMut<'w, EditMode>,
    pub learn_ability: ResMut<'w, LearnAbilityOpen>,
    pub learn_trait: ResMut<'w, LearnTraitOpen>,
    pub create_item: ResMut<'w, crate::create_item::CreateItemOpen>,
}

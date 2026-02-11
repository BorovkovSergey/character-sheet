use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR};
use ui_widgets::composites::{
    Abilities, AbilityEntry, AddItemMenu, AddItemSelection, Characteristics, EquippedGear,
    IdentityBar, Inventory, Points, Portrait, SkillEntry, Skills, Stats, StatusBar,
    StatusBarResponse, TraitEntry, Traits, Wallet as WalletWidget, WalletResponse, Weapon,
    WeaponSlot,
};
use ui_widgets::molecules::{AbilityCard, CellAction, InventoryTooltip, SmallAbility};
use ui_widgets::styles::UiStyle;

use crate::components::{
    AbilityPoints, ActionPoints, ActiveCharacter, ActiveEffects, CharacterAbilityNames,
    CharacterClass, CharacterEquipment, CharacterName, CharacterRace, CharacterSkillList,
    CharacterStats, CharacterTraitNames, CharacterWeaponNames, CharacteristicPoints, Experience,
    Hp, Inventory as InventoryComponent, Level, Mana, SkillPoints, TraitPoints, Wallet,
};
use crate::events::{
    CreateItem, ExperienceChanged, InventoryChanged, LearnAbility, LearnTrait, LevelUp,
    ResourceChanged, UpgradeEvent, WalletChanged,
};
use crate::state::AppScreen;
use shared::character::OnLvlUp;
use shared::{
    AbilityCheck, AbilityType, CharacterTrait, CharacteristicKind, CharacteristicTrait, Effect,
    EquipmentSlot, InventoryItem,
};

#[derive(Resource)]
struct UiIcons {
    heart: egui::TextureHandle,
    avatar_border_1: egui::TextureHandle,
    avatar_border_2: egui::TextureHandle,
    avatar_placeholder: egui::TextureHandle,
    wallet_gold: egui::TextureHandle,
    wallet_silver: egui::TextureHandle,
    wallet_copper: egui::TextureHandle,
    ability_placeholder: egui::TextureHandle,
    weapon_placeholder: egui::TextureHandle,
    inventory_placeholder: egui::TextureHandle,
    shield: egui::TextureHandle,
}

fn load_png_texture(ctx: &egui::Context, name: &str, png_bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(png_bytes).expect("failed to decode PNG");
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

#[derive(Resource, Default)]
pub struct EditMode(pub bool);

#[derive(Resource, Default)]
pub struct LearnAbilityOpen(pub bool);

#[derive(Resource, Default)]
pub struct LearnTraitOpen(pub bool);

#[derive(SystemParam)]
struct UiEvents<'w> {
    resource: MessageWriter<'w, ResourceChanged>,
    wallet: MessageWriter<'w, WalletChanged>,
    inventory: MessageWriter<'w, InventoryChanged>,
    experience: MessageWriter<'w, ExperienceChanged>,
    upgrade: MessageWriter<'w, UpgradeEvent>,
    learn_ability: MessageWriter<'w, LearnAbility>,
    learn_trait: MessageWriter<'w, LearnTrait>,
    create_item: MessageWriter<'w, CreateItem>,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditMode>()
            .init_resource::<LearnAbilityOpen>()
            .init_resource::<LearnTraitOpen>()
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
                    init_icons.run_if(not(resource_exists::<UiIcons>)),
                    render_ui.run_if(in_state(AppScreen::CharacterSheet)),
                ),
            )
            .add_systems(
                Update,
                (
                    apply_resource_changes,
                    apply_wallet_changes,
                    apply_inventory_changes,
                    apply_experience_changes,
                    apply_level_up,
                    apply_upgrades,
                    apply_learn_ability,
                    apply_learn_trait,
                ),
            )
            .add_systems(Update, apply_create_item);
    }
}

fn init_icons(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    UiStyle::apply_global_style(ctx);
    commands.insert_resource(UiIcons {
        heart: load_png_texture(ctx, "heart", include_bytes!("../assets/heart.png")),
        avatar_border_1: load_png_texture(
            ctx,
            "avatar_border_1",
            include_bytes!("../assets/avatar_border_1.png"),
        ),
        avatar_border_2: load_png_texture(
            ctx,
            "avatar_border_2",
            include_bytes!("../assets/avatar_border_2.png"),
        ),
        avatar_placeholder: load_png_texture(
            ctx,
            "avatar_placeholder",
            include_bytes!("../assets/avatar_placeholder.png"),
        ),
        wallet_gold: load_png_texture(
            ctx,
            "wallet_gold",
            include_bytes!("../assets/wallet_gold.png"),
        ),
        wallet_silver: load_png_texture(
            ctx,
            "wallet_silver",
            include_bytes!("../assets/wallet_silver.png"),
        ),
        wallet_copper: load_png_texture(
            ctx,
            "wallet_copper",
            include_bytes!("../assets/wallet_copper.png"),
        ),
        ability_placeholder: load_png_texture(
            ctx,
            "ability_placeholder",
            include_bytes!("../assets/ph_ability.png"),
        ),
        weapon_placeholder: load_png_texture(
            ctx,
            "weapon_placeholder",
            include_bytes!("../assets/ph_weapon.png"),
        ),
        inventory_placeholder: load_png_texture(
            ctx,
            "inventory_placeholder",
            include_bytes!("../assets/ph_inventory.png"),
        ),
        shield: load_png_texture(ctx, "shield", include_bytes!("../assets/shield.png")),
    });
    Ok(())
}

const MARGIN: f32 = 0.02;
const COL_GAP: f32 = 0.01;
const COL1_WIDTH: f32 = 0.24;
const COL2_WIDTH: f32 = 0.46;
const COL3_WIDTH: f32 = 0.24;

#[derive(bevy::ecs::query::QueryData)]
struct CharacterQueryData {
    name: &'static CharacterName,
    race: &'static CharacterRace,
    class: &'static CharacterClass,
    level: &'static Level,
    exp: &'static Experience,
    hp: &'static Hp,
    mana: &'static Mana,
    ap: &'static ActionPoints,
    stats: &'static CharacterStats,
    trait_names: &'static CharacterTraitNames,
    ability_names: &'static CharacterAbilityNames,
    char_pts: &'static CharacteristicPoints,
    skill_pts: &'static SkillPoints,
    ability_pts: &'static AbilityPoints,
    trait_pts: &'static TraitPoints,
    skills: &'static CharacterSkillList,
    wallet: &'static Wallet,
    weapon_names: &'static CharacterWeaponNames,
    equipment: &'static CharacterEquipment,
    inventory: &'static InventoryComponent,
    effects: &'static ActiveEffects,
}

fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    character_query: Query<CharacterQueryData, With<ActiveCharacter>>,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
    skill_registry: Res<crate::network::ClientSkillRegistry>,
    ability_registry: Res<crate::network::ClientAbilityRegistry>,
    weapon_registry: Res<crate::network::ClientWeaponRegistry>,
    equipment_registry: Res<crate::network::ClientEquipmentRegistry>,
    item_registry: Res<crate::network::ClientItemRegistry>,
    mut ui_events: UiEvents,
    mut edit_mode: ResMut<EditMode>,
    mut learn_ability_open: ResMut<LearnAbilityOpen>,
    mut learn_trait_open: ResMut<LearnTraitOpen>,
    mut create_item_open: ResMut<crate::create_item::CreateItemOpen>,
) -> Result {
    let Some(icons) = icons else {
        return Ok(());
    };

    let Ok(character) = character_query.single() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |ui| {
            let total_w = ui.available_width();
            let total_h = ui.available_height();

            let margin = total_w * MARGIN;
            let gap = total_w * COL_GAP;
            let top_margin = margin / 2.0;
            let bottom_margin = top_margin;
            let col_h = total_h - top_margin - bottom_margin;

            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.add_space(top_margin);

            ui.horizontal(|ui| {
                ui.add_space(margin);
                render_left_column(
                    ui,
                    total_w * COL1_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &weapon_registry,
                    &equipment_registry,
                    &item_registry,
                    &mut ui_events,
                    &mut edit_mode,
                    &mut learn_ability_open,
                    &mut learn_trait_open,
                    &mut create_item_open,
                );
                ui.add_space(gap);
                render_center_column(
                    ui,
                    total_w * COL2_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &trait_registry,
                    &skill_registry,
                    &ability_registry,
                    &mut ui_events,
                    edit_mode.0,
                );
                ui.add_space(gap);
                render_right_column(
                    ui,
                    total_w * COL3_WIDTH,
                    col_h,
                    &icons,
                    &character,
                    &weapon_registry,
                    &equipment_registry,
                    &item_registry,
                    &mut ui_events,
                );
            });
        });

    // "Learn Ability" overlay
    if learn_ability_open.0 {
        let screen = ctx.content_rect();

        // Semi-transparent backdrop that blocks interaction behind the dialog
        egui::Area::new(egui::Id::new("learn_ability_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(120));
                if resp.clicked() {
                    learn_ability_open.0 = false;
                }
            });

        // Centered dialog (half width, half height)
        let dialog_size = screen.size() * 0.5;
        let dialog_pos = egui::pos2(
            screen.center().x - dialog_size.x / 2.0,
            screen.center().y - dialog_size.y / 2.0,
        );
        egui::Area::new(egui::Id::new("learn_ability_dialog"))
            .order(egui::Order::Foreground)
            .fixed_pos(dialog_pos)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(dialog_size, egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(16), MAIN_COLOR);
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::same(16),
                    egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
                    egui::StrokeKind::Inside,
                );

                let pad = rect.width() * 0.04;
                let content = rect.shrink(pad);

                // 3 rows in staggered pattern: 3, 2, 3
                let rows: [usize; 3] = [3, 2, 3];
                let gap = content.width() * 0.03;
                let cell_w = (content.width() - gap * 2.0) / 3.0;
                let cell_h = (content.height() - gap * 2.0) / 3.0;
                let half_offset = (cell_w + gap) / 2.0;

                // Build a grid of ability data from the registry.
                // grid[row][col] — row maps directly to LearnScreenPosition.row,
                // col maps to LearnScreenPosition.column.
                // Tuple: (name, mp_cost, can_learn, already_learned)
                let mut grid: [[Option<(&str, Option<u32>, bool, bool)>; 3]; 3] =
                    Default::default();
                if let Some(class_abilities) =
                    ability_registry.0.get_class_abilities(&character.class.0)
                {
                    let known = &character.ability_names.0;
                    for (name, ability) in &class_abilities.acquire {
                        if let Some(pos) = &ability.learn_screen_position {
                            let r = pos.row as usize;
                            let c = pos.column as usize;
                            if r < 3 && c < 3 {
                                let mp = ability.requirements.as_ref().and_then(|r| r.mp);
                                let already_learned = known.contains(name);
                                let can_learn = ability.can_learn_after.is_empty()
                                    || ability
                                        .can_learn_after
                                        .iter()
                                        .any(|prereq| known.contains(prereq));
                                grid[r][c] = Some((name.as_str(), mp, can_learn, already_learned));
                            }
                        }
                    }
                }

                let ability_icon = icons.ability_placeholder.id();
                let class_abilities = ability_registry.0.get_class_abilities(&character.class.0);
                for (row_idx, &col_count) in rows.iter().enumerate() {
                    let y = content.min.y + (cell_h + gap) * row_idx as f32;
                    let x_offset = if col_count == 2 { half_offset } else { 0.0 };
                    for col in 0..col_count {
                        let x = content.min.x + x_offset + (cell_w + gap) * col as f32;
                        let cell_rect =
                            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_w, cell_h));
                        if let Some((name, mp, can_learn, learned)) = grid[row_idx][col] {
                            let fill = if can_learn {
                                MAIN_COLOR
                            } else {
                                SECONDARY_COLOR
                            };
                            SmallAbility::new(name, ability_icon)
                                .mp_cost(mp)
                                .fill(fill)
                                .learned(learned)
                                .paint(ui.painter(), cell_rect);

                            let cell_id = egui::Id::new("learn_cell").with(row_idx).with(col);
                            let response =
                                ui.interact(cell_rect, cell_id, egui::Sense::click_and_drag());
                            if can_learn
                                && !learned
                                && character.ability_pts.0 > 0
                                && response.clicked()
                            {
                                ui_events
                                    .learn_ability
                                    .write(LearnAbility(name.to_string()));
                                if character.ability_pts.0 == 1 {
                                    learn_ability_open.0 = false;
                                }
                            }
                            if response.hovered() {
                                if let Some(ability) =
                                    class_abilities.and_then(|ca| ca.acquire.get(name))
                                {
                                    let card_w = cell_rect.width() * 1.8;
                                    let card_h = cell_rect.height() * 1.5;
                                    let card_pos = egui::pos2(
                                        cell_rect.center().x - card_w / 2.0,
                                        cell_rect.min.y - card_h - 8.0,
                                    );
                                    egui::Area::new(cell_id.with("tooltip"))
                                        .order(egui::Order::Tooltip)
                                        .fixed_pos(card_pos)
                                        .show(ui.ctx(), |ui| {
                                            let (card_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(card_w, card_h),
                                                egui::Sense::hover(),
                                            );
                                            AbilityCard::new(ability_icon, &ability.description)
                                                .name(name)
                                                .mp_cost(
                                                    ability
                                                        .requirements
                                                        .as_ref()
                                                        .and_then(|r| r.mp),
                                                )
                                                .ap_cost(
                                                    ability
                                                        .requirements
                                                        .as_ref()
                                                        .and_then(|r| r.action_points),
                                                )
                                                .self_only(ability.self_only)
                                                .range(
                                                    ability
                                                        .requirements
                                                        .as_ref()
                                                        .and_then(|r| r.range),
                                                )
                                                .ability_type(format_ability_type(
                                                    ability.ability_type,
                                                ))
                                                .check(
                                                    ability
                                                        .check
                                                        .as_ref()
                                                        .map(format_ability_check)
                                                        .unwrap_or_default(),
                                                )
                                                .enemy_check(
                                                    ability
                                                        .enemy_check
                                                        .as_ref()
                                                        .map(|e| e.to_string())
                                                        .unwrap_or_default(),
                                                )
                                                .paint(ui.painter(), card_rect);
                                            ui.painter().rect_stroke(
                                                card_rect,
                                                egui::CornerRadius::same(12),
                                                egui::Stroke::new(
                                                    1.0,
                                                    egui::Color32::from_gray(200),
                                                ),
                                                egui::StrokeKind::Inside,
                                            );
                                        });
                                }
                            }
                        } else {
                            SmallAbility::new("", ability_icon).paint(ui.painter(), cell_rect);
                        }
                    }
                }
            });
    }

    // "Learn Trait" overlay
    if learn_trait_open.0 {
        render_learn_trait_overlay(
            ctx,
            &character,
            &trait_registry,
            &mut ui_events.learn_trait,
            &mut learn_trait_open,
        );
    }

    // "Create Item" overlay
    if create_item_open.0 {
        let skill_names: Vec<String> = skill_registry
            .0
            .classes
            .values()
            .flat_map(|skills| skills.keys().cloned())
            .collect();
        crate::create_item::render_create_item_popup(
            ctx,
            &mut create_item_open,
            &mut ui_events.create_item,
            &format_effect,
            &skill_names,
        );
    }

    Ok(())
}

fn render_left_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    weapon_registry: &crate::network::ClientWeaponRegistry,
    equipment_registry: &crate::network::ClientEquipmentRegistry,
    item_registry: &crate::network::ClientItemRegistry,
    ui_events: &mut UiEvents,
    edit_mode: &mut EditMode,
    learn_ability_open: &mut LearnAbilityOpen,
    learn_trait_open: &mut LearnTraitOpen,
    create_item_open: &mut crate::create_item::CreateItemOpen,
) {
    let gap = height * 0.03 / 4.0;
    let initiative =
        character.stats.0.perception.level as i32 + character.effects.initiative_bonus();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let portrait_size = egui::vec2(width, height * 0.30);
        let (portrait_rect, _) = ui.allocate_exact_size(portrait_size, egui::Sense::hover());
        let mut portrait_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(portrait_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let add_item_menu =
            build_add_item_menu(&weapon_registry.0, &equipment_registry.0, &item_registry.0);
        let portrait_resp = Portrait::new(
            icons.avatar_border_1.id(),
            icons.avatar_border_2.id(),
            icons.avatar_placeholder.id(),
            character.level.0,
            character.exp.0,
            edit_mode.0,
        )
        .shield(icons.shield.id(), character.effects.armor())
        .ability_points(character.ability_pts.0)
        .trait_points(character.trait_pts.0)
        .add_item_menu(add_item_menu)
        .show(&mut portrait_ui);
        if let Some(exp) = portrait_resp.add_exp {
            ui_events.experience.write(ExperienceChanged(exp));
        }
        if portrait_resp.toggle_edit {
            edit_mode.0 = !edit_mode.0;
        }
        if portrait_resp.open_learn_ability {
            learn_ability_open.0 = true;
        }
        if portrait_resp.open_learn_trait {
            learn_trait_open.0 = true;
        }
        if portrait_resp.open_create_item {
            create_item_open.0 = true;
        }
        if let Some(selection) = portrait_resp.add_item {
            let inv_item = match selection {
                AddItemSelection::Item(name) => InventoryItem::Item(name),
                AddItemSelection::Equipment(name) => InventoryItem::Equipment(name),
                AddItemSelection::Weapon(name) => InventoryItem::Weapon(name),
            };
            ui_events
                .inventory
                .write(InventoryChanged::AddExisting(inv_item));
        }
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.11],
            IdentityBar::new(
                &character.name.0,
                &character.race.0.to_string(),
                &character.class.0.to_string(),
            ),
        );
        ui.add_space(gap);

        send_status_bar_events(
            ui,
            width,
            height * 0.16,
            character,
            initiative,
            &mut ui_events.resource,
        );
        ui.add_space(gap);

        let resists = character
            .effects
            .get_resists()
            .into_iter()
            .map(|(r, v)| (r.to_string(), v))
            .collect();
        let protections = character
            .effects
            .get_protections()
            .into_iter()
            .map(|(p, v)| (p.to_string(), v))
            .collect();
        ui.add_sized(
            [width, height * 0.20],
            Stats::new(icons.heart.id(), resists, protections),
        );
        ui.add_space(gap);

        let weapon_slots: Vec<WeaponSlot> = character
            .weapon_names
            .0
            .iter()
            .filter_map(|name| {
                weapon_registry.0.get(name).map(|w| WeaponSlot {
                    name: w.name.clone(),
                    kind: w.kind.to_string(),
                    attack: format!("{:+}", w.attack),
                    damage: w.damage.clone(),
                    range: w.range.to_string(),
                    condition: w.condition.clone().unwrap_or_default(),
                })
            })
            .collect();

        let weapon_size = egui::vec2(width, height * 0.20);
        let (weapon_rect, _) = ui.allocate_exact_size(weapon_size, egui::Sense::hover());
        let mut weapon_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(weapon_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(i) =
            Weapon::new(icons.weapon_placeholder.id(), weapon_slots).show(&mut weapon_ui)
        {
            ui_events
                .inventory
                .write(InventoryChanged::UnequipWeapon(i));
        }
    });
}

fn build_add_item_menu(
    weapon_registry: &shared::WeaponRegistry,
    equipment_registry: &shared::EquipmentRegistry,
    item_registry: &shared::ItemRegistry,
) -> AddItemMenu {
    use std::collections::BTreeMap;
    use ui_widgets::molecules::InventoryTooltip;

    let items: Vec<InventoryTooltip> = item_registry
        .items
        .values()
        .map(|i| InventoryTooltip::Item {
            name: i.name.clone(),
            description: i.description.clone(),
        })
        .collect();

    let mut equipment: BTreeMap<String, Vec<InventoryTooltip>> = BTreeMap::new();
    for eq in equipment_registry.equipment.values() {
        equipment
            .entry(eq.slot.to_string())
            .or_default()
            .push(InventoryTooltip::Equipment {
                name: eq.name.clone(),
                slot: eq.slot.to_string(),
                description: eq.description.clone(),
                effects: eq.effects.iter().map(format_effect).collect(),
            });
    }

    let mut weapons: BTreeMap<String, Vec<InventoryTooltip>> = BTreeMap::new();
    for w in weapon_registry.weapons.values() {
        weapons
            .entry(w.kind.to_string())
            .or_default()
            .push(InventoryTooltip::Weapon {
                name: w.name.clone(),
                kind: w.kind.to_string(),
                attack: format!("{:+}", w.attack),
                damage: w.damage.clone(),
                range: w.range.to_string(),
                condition: w.condition.clone().unwrap_or_default(),
            });
    }

    AddItemMenu {
        items,
        equipment,
        weapons,
    }
}

fn send_status_bar_events(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    character: &CharacterQueryDataItem,
    initiative: i32,
    events: &mut MessageWriter<ResourceChanged>,
) {
    let status_size = egui::vec2(width, height);
    let (status_rect, _) = ui.allocate_exact_size(status_size, egui::Sense::hover());
    let mut status_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(status_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let result = StatusBar::new(
        character.hp.current,
        character.hp.max,
        character.mana.current,
        character.mana.max,
        character.ap.current,
        character.ap.max,
        initiative,
    )
    .show(&mut status_ui);

    send_resource_events(events, result);
}

fn send_resource_events(events: &mut MessageWriter<ResourceChanged>, result: StatusBarResponse) {
    if let Some(v) = result.hp {
        events.write(ResourceChanged::Hp(v));
    }
    if let Some(v) = result.mp {
        events.write(ResourceChanged::Mp(v));
    }
    if let Some(v) = result.ap {
        events.write(ResourceChanged::Ap(v));
    }
}

/// Applies resource change messages to the active character's ECS components.
fn apply_resource_changes(
    mut query: Query<(&mut Hp, &mut Mana, &mut ActionPoints), With<ActiveCharacter>>,
    mut reader: MessageReader<ResourceChanged>,
) {
    let Ok((mut hp, mut mana, mut ap)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        match event {
            ResourceChanged::Hp(v) => hp.current = (*v).min(hp.max),
            ResourceChanged::Mp(v) => mana.current = (*v).min(mana.max),
            ResourceChanged::Ap(v) => ap.current = (*v).min(ap.max),
        }
    }
}

/// Applies experience change messages to the active character's ECS components.
fn apply_experience_changes(
    mut query: Query<(&mut Experience, &mut Level), With<ActiveCharacter>>,
    mut reader: MessageReader<ExperienceChanged>,
    mut level_up: MessageWriter<LevelUp>,
) {
    let Ok((mut exp, mut level)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        exp.0 += event.0;
        // Level up: subtract (next_level * 10) XP each time the threshold is met.
        loop {
            let needed = (level.0 + 1) * 10;
            if exp.0 < needed {
                break;
            }
            exp.0 -= needed;
            level.0 += 1;
            level_up.write(LevelUp);
        }
    }
}

/// Applies all OnLvlUp effects from active effects on level up.
/// Also grants 1 trait point every 3 levels.
fn apply_level_up(
    mut query: Query<
        (
            &Level,
            &ActiveEffects,
            &mut AbilityPoints,
            &mut SkillPoints,
            &mut CharacteristicPoints,
            &mut TraitPoints,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<LevelUp>,
) {
    let Ok((level, effects, mut ability_pts, mut skill_pts, mut char_pts, mut trait_pts)) =
        query.single_mut()
    else {
        return;
    };
    let level_ups: u32 = reader.read().count() as u32;
    if level_ups == 0 {
        return;
    }
    for _ in 0..level_ups {
        for effect in &effects.0 {
            if let Effect::OnLvlUp(on_lvl_up) = effect {
                match on_lvl_up {
                    OnLvlUp::AddAbilityPoints(v) => {
                        ability_pts.0 = (ability_pts.0 as i32 + v).max(0) as u32;
                    }
                    OnLvlUp::AddSkillPoints(v) => {
                        skill_pts.0 = (skill_pts.0 as i32 + v).max(0) as u32;
                    }
                    OnLvlUp::AddCharacteristicPoints(v) => {
                        char_pts.0 = (char_pts.0 as i32 + v).max(0) as u32;
                    }
                }
            }
        }
    }
    // Grant 1 trait point for each level divisible by 3 that was crossed.
    let prev_level = level.0 - level_ups;
    trait_pts.0 += level.0 / 3 - prev_level / 3;
}

/// Applies characteristic and skill upgrades from edit mode.
fn apply_upgrades(
    mut query: Query<
        (
            &CharacterClass,
            &mut CharacterStats,
            &mut CharacteristicPoints,
            &mut SkillPoints,
            &mut CharacterSkillList,
            &ActiveEffects,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<UpgradeEvent>,
    skill_registry: Res<crate::network::ClientSkillRegistry>,
) {
    let Ok((class, mut stats, mut char_pts, mut skill_pts, mut skills, effects)) =
        query.single_mut()
    else {
        return;
    };

    let char_kinds = [
        CharacteristicKind::Strength,
        CharacteristicKind::Dexterity,
        CharacteristicKind::Endurance,
        CharacteristicKind::Perception,
        CharacteristicKind::Magic,
        CharacteristicKind::Willpower,
        CharacteristicKind::Intellect,
        CharacteristicKind::Charisma,
    ];

    let char_bonuses = effects.characteristic_bonuses();

    for event in reader.read() {
        match event {
            UpgradeEvent::Characteristic(idx) => {
                let s = &mut stats.0;
                let cost = match char_kinds.get(*idx) {
                    Some(CharacteristicKind::Strength) => s.strength.up(char_pts.0),
                    Some(CharacteristicKind::Dexterity) => s.dexterity.up(char_pts.0),
                    Some(CharacteristicKind::Endurance) => s.endurance.up(char_pts.0),
                    Some(CharacteristicKind::Perception) => s.perception.up(char_pts.0),
                    Some(CharacteristicKind::Magic) => s.magic.up(char_pts.0),
                    Some(CharacteristicKind::Willpower) => s.willpower.up(char_pts.0),
                    Some(CharacteristicKind::Intellect) => s.intellect.up(char_pts.0),
                    Some(CharacteristicKind::Charisma) => s.charisma.up(char_pts.0),
                    None => 0,
                };
                char_pts.0 -= cost;
            }
            UpgradeEvent::Skill(name) => {
                let max_level = skill_registry
                    .0
                    .get_skill(&class.0, name)
                    .map(|skill| {
                        let base = stats.0.get_level(skill.dependency);
                        let bonus = char_bonuses.get(&skill.dependency).copied().unwrap_or(0);
                        (base as i32 + bonus).max(0) as u32
                    })
                    .unwrap_or(0);

                let char_skill = skills.0.iter_mut().find(|s| s.name == *name);
                if let Some(skill) = char_skill {
                    let cost = skill.up(skill_pts.0, max_level);
                    skill_pts.0 -= cost;
                } else if skill_pts.0 >= 1 && max_level >= 1 {
                    skills.0.push(shared::CharacterSkill {
                        name: name.clone(),
                        level: 1,
                    });
                    skill_pts.0 -= 1;
                }
            }
        }
    }
}

/// Learns an ability: adds it to the character's ability list and deducts one ability point.
fn apply_learn_ability(
    mut query: Query<(&mut CharacterAbilityNames, &mut AbilityPoints), With<ActiveCharacter>>,
    mut reader: MessageReader<LearnAbility>,
) {
    let Ok((mut abilities, mut pts)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        if pts.0 > 0 && !abilities.0.contains(&event.0) {
            abilities.0.push(event.0.clone());
            pts.0 -= 1;
        }
    }
}

/// Learns a trait: validates conditions, adds it to the character's trait list and deducts one trait point.
fn apply_learn_trait(
    mut query: Query<
        (&mut CharacterTraitNames, &mut TraitPoints, &CharacterStats),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<LearnTrait>,
    trait_registry: Res<crate::network::ClientTraitRegistry>,
) {
    let Ok((mut traits, mut pts, stats)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        if pts.0 == 0 || traits.0.contains(&event.0) {
            continue;
        }
        let Some(ct) = trait_registry.0.get(&event.0) else {
            continue;
        };
        if !check_trait_requirement(&stats.0, ct.condition.as_ref()) {
            continue;
        }
        traits.0.push(event.0.clone());
        pts.0 -= 1;
    }
}

fn check_trait_requirement(
    stats: &shared::Characteristics,
    condition: Option<&shared::TraitCondition>,
) -> bool {
    match condition {
        Some(shared::TraitCondition::CharacteristicsRequired {
            characteristic,
            lvl,
        }) => {
            let char_level = match characteristic {
                CharacteristicKind::Strength => stats.strength.level,
                CharacteristicKind::Dexterity => stats.dexterity.level,
                CharacteristicKind::Endurance => stats.endurance.level,
                CharacteristicKind::Perception => stats.perception.level,
                CharacteristicKind::Magic => stats.magic.level,
                CharacteristicKind::Willpower => stats.willpower.level,
                CharacteristicKind::Intellect => stats.intellect.level,
                CharacteristicKind::Charisma => stats.charisma.level,
            };
            char_level >= *lvl
        }
        None => true,
    }
}

fn render_center_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    trait_registry: &crate::network::ClientTraitRegistry,
    skill_registry: &crate::network::ClientSkillRegistry,
    ability_registry: &crate::network::ClientAbilityRegistry,
    ui_events: &mut UiEvents,
    edit_mode: bool,
) {
    let gap = height * 0.03 / 4.0;
    let stats = &character.stats.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let characteristics = [
            ("STR", stats.strength.level),
            ("DEX", stats.dexterity.level),
            ("END", stats.endurance.level),
            ("PER", stats.perception.level),
            ("MAG", stats.magic.level),
            ("WIL", stats.willpower.level),
            ("INT", stats.intellect.level),
            ("CHA", stats.charisma.level),
        ];
        let char_values = characteristics
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        let char_size = egui::vec2(width, height * 0.14);
        let (char_rect, _) = ui.allocate_exact_size(char_size, egui::Sense::hover());
        let mut char_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(char_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(idx) = Characteristics::new(char_values)
            .edit_mode(edit_mode, character.char_pts.0)
            .show(&mut char_ui)
        {
            ui_events.upgrade.write(UpgradeEvent::Characteristic(idx));
        }
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.05],
            Points::new(character.char_pts.0, character.skill_pts.0),
        );
        ui.add_space(gap);
        let char_bonuses = character.effects.characteristic_bonuses();
        let effective_level = |kind: CharacteristicKind| -> u32 {
            let base = stats.get_level(kind);
            let bonus = char_bonuses.get(&kind).copied().unwrap_or(0);
            (base as i32 + bonus).max(0) as u32
        };

        let skill_entries: Vec<SkillEntry> = skill_registry
            .0
            .get_class_skills(&character.class.0)
            .into_iter()
            .flat_map(|skills| skills.iter())
            .map(|(name, skill)| {
                let base_level = character
                    .skills
                    .0
                    .iter()
                    .find(|s| s.name == *name)
                    .map_or(0, |s| s.level);
                let skill_bonus = character.effects.skill_bonus(name);
                let max_level = effective_level(skill.dependency);
                SkillEntry {
                    name: name.clone(),
                    dependency: skill.dependency.to_string(),
                    level: base_level as i32 + skill_bonus,
                    max_level,
                }
            })
            .collect();

        let skill_size = egui::vec2(width, height * 0.24);
        let (skill_rect, _) = ui.allocate_exact_size(skill_size, egui::Sense::hover());
        let mut skill_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(skill_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(idx) = Skills::new(skill_entries)
            .edit_mode(edit_mode, character.skill_pts.0)
            .show(&mut skill_ui)
        {
            if let Some(name) = skill_registry
                .0
                .get_class_skills(&character.class.0)
                .into_iter()
                .flat_map(|skills| skills.keys())
                .nth(idx)
            {
                ui_events.upgrade.write(UpgradeEvent::Skill(name.clone()));
            }
        }
        ui.add_space(gap);
        let trait_entries: Vec<TraitEntry> = character
            .trait_names
            .0
            .iter()
            .filter_map(|name| {
                trait_registry.0.get(name).map(|ct| TraitEntry {
                    name: name.clone(),
                    description: ct.description.clone(),
                    effects: ct.effects.iter().map(format_effect).collect(),
                })
            })
            .collect();
        ui.add_sized([width, height * 0.14], Traits::new(trait_entries));
        ui.add_space(gap);
        let ability_entries: Vec<AbilityEntry> = character
            .ability_names
            .0
            .iter()
            .filter_map(|name| {
                let ability = ability_registry
                    .0
                    .get_class_abilities(&character.class.0)
                    .and_then(|ca| ca.innate.get(name).or_else(|| ca.acquire.get(name)));
                ability.map(|a| AbilityEntry {
                    name: name.clone(),
                    description: a.description.clone(),
                    image: icons.ability_placeholder.id(),
                    mp_cost: a.requirements.as_ref().and_then(|r| r.mp),
                    ap_cost: a.requirements.as_ref().and_then(|r| r.action_points),
                    self_only: a.self_only,
                    range: a.requirements.as_ref().and_then(|r| r.range),
                    ability_type: format_ability_type(a.ability_type),
                    check: a
                        .check
                        .as_ref()
                        .map(format_ability_check)
                        .unwrap_or_default(),
                    enemy_check: a
                        .enemy_check
                        .as_ref()
                        .map(|e| e.to_string())
                        .unwrap_or_default(),
                })
            })
            .collect();
        let abilities_size = egui::vec2(width, height * 0.40);
        let (abilities_rect, _) = ui.allocate_exact_size(abilities_size, egui::Sense::hover());
        let mut abilities_ui = ui.new_child(egui::UiBuilder::new().max_rect(abilities_rect));
        if let Some(new_mp) =
            Abilities::new(ability_entries, character.mana.current).show(&mut abilities_ui)
        {
            ui_events.resource.write(ResourceChanged::Mp(new_mp));
        }
    });
}

/// State stored in egui temp data for the "Learn Trait" dialog.
#[derive(Clone, Default)]
struct LearnTraitDialogState {
    selected: Option<String>,
}

fn render_learn_trait_overlay(
    ctx: &egui::Context,
    character: &CharacterQueryDataItem,
    trait_registry: &crate::network::ClientTraitRegistry,
    learn_trait_events: &mut MessageWriter<LearnTrait>,
    learn_trait_open: &mut LearnTraitOpen,
) {
    let screen = ctx.content_rect();
    let state_id = egui::Id::new("learn_trait_state");

    // Semi-transparent backdrop
    egui::Area::new(egui::Id::new("learn_trait_backdrop"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(120));
            if resp.clicked() {
                learn_trait_open.0 = false;
                ctx.data_mut(|d| d.remove::<LearnTraitDialogState>(state_id));
            }
        });

    // Centered dialog
    let dialog_size = egui::vec2(screen.width() * 0.4, screen.height() * 0.6);
    let dialog_pos = egui::pos2(
        screen.center().x - dialog_size.x / 2.0,
        screen.center().y - dialog_size.y / 2.0,
    );

    egui::Area::new(egui::Id::new("learn_trait_dialog"))
        .order(egui::Order::Foreground)
        .fixed_pos(dialog_pos)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(dialog_size, egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(16), MAIN_COLOR);
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(16),
                egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
                egui::StrokeKind::Inside,
            );

            let pad = 16.0;
            let content = rect.shrink(pad);

            // Title
            let title_height = 30.0;
            let title_rect =
                egui::Rect::from_min_size(content.min, egui::vec2(content.width(), title_height));
            ui.painter().text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Learn Trait",
                egui::FontId::proportional(20.0),
                ui_widgets::colors::TEXT_COLOR,
            );

            // Buttons area at the bottom
            let button_height = 36.0;
            let button_area_top = content.max.y - button_height;

            // Scrollable list area
            let list_top = content.min.y + title_height + 8.0;
            let list_rect = egui::Rect::from_min_max(
                egui::pos2(content.min.x, list_top),
                egui::pos2(content.max.x, button_area_top - 8.0),
            );

            let mut state: LearnTraitDialogState =
                ctx.data(|d| d.get_temp(state_id)).unwrap_or_default();

            let known_traits = &character.trait_names.0;

            // Build sorted list of all traits
            let mut all_traits: Vec<(&String, &CharacterTrait)> =
                trait_registry.0.traits.iter().collect();
            all_traits.sort_by_key(|(name, _)| (*name).clone());

            // Scrollable list with radio buttons
            let mut list_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(list_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            egui::ScrollArea::vertical()
                .id_salt("learn_trait_scroll")
                .auto_shrink(false)
                .show(&mut list_ui, |ui| {
                    for (name, ct) in &all_traits {
                        let already_learned = known_traits.contains(name);
                        let meets_requirement =
                            check_trait_requirement(&character.stats.0, ct.condition.as_ref());
                        let available = !already_learned && meets_requirement;
                        let is_selected = state.selected.as_deref() == Some(name.as_str());

                        ui.horizontal(|ui| {
                            if !available {
                                ui.disable();
                            }

                            if ui.radio(is_selected, "").clicked() && available {
                                state.selected = Some((*name).clone());
                            }

                            ui.vertical(|ui| {
                                let label = if already_learned {
                                    format!("{} (learned)", name)
                                } else {
                                    (*name).clone()
                                };
                                ui.label(
                                    egui::RichText::new(label)
                                        .strong()
                                        .size(14.0)
                                        .color(ui_widgets::colors::TEXT_COLOR),
                                );
                                ui.label(
                                    egui::RichText::new(&ct.description)
                                        .size(12.0)
                                        .color(egui::Color32::GRAY),
                                );
                                if !ct.effects.is_empty() {
                                    let effects_text: Vec<String> =
                                        ct.effects.iter().map(format_effect).collect();
                                    ui.label(
                                        egui::RichText::new(effects_text.join(", "))
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(0x00, 0x99, 0x66)),
                                    );
                                }
                                if let Some(shared::TraitCondition::CharacteristicsRequired {
                                    characteristic,
                                    lvl,
                                }) = &ct.condition
                                {
                                    let color = if meets_requirement {
                                        egui::Color32::from_rgb(0x99, 0x66, 0x00)
                                    } else {
                                        egui::Color32::from_rgb(0xCC, 0x33, 0x33)
                                    };
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Requires: {} {}",
                                            characteristic, lvl
                                        ))
                                        .size(11.0)
                                        .italics()
                                        .color(color),
                                    );
                                }
                            });
                        });
                        ui.add_space(4.0);
                    }
                });

            // Buttons
            let button_rect = egui::Rect::from_min_size(
                egui::pos2(content.min.x, button_area_top),
                egui::vec2(content.width(), button_height),
            );
            let mut button_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(button_rect)
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
            );

            let can_confirm = state.selected.as_ref().is_some_and(|name| {
                character.trait_pts.0 > 0 && !known_traits.contains(name)
            });
            if button_ui
                .add_enabled(can_confirm, egui::Button::new("Confirm"))
                .clicked()
            {
                if let Some(name) = &state.selected {
                    if !known_traits.contains(name) {
                        learn_trait_events.write(LearnTrait(name.clone()));
                    }
                }
                learn_trait_open.0 = false;
                ctx.data_mut(|d| d.remove::<LearnTraitDialogState>(state_id));
            }
            if button_ui.button("Cancel").clicked() {
                learn_trait_open.0 = false;
                ctx.data_mut(|d| d.remove::<LearnTraitDialogState>(state_id));
            }

            ctx.data_mut(|d| d.insert_temp(state_id, state));
        });
}

fn format_effect(effect: &Effect) -> String {
    match effect {
        Effect::Resist(r, v) => format!("{r} Resist +{v}"),
        Effect::Skill(name, v) => format!("{name} {v:+}"),
        Effect::Protection(p, v) => format!("{p} Protection +{v}"),
        Effect::Initiative(v) => format!("Initiative {v:+}"),
        Effect::Characteristic(c, v) => format!("{c} {v:+}"),
        Effect::ActionPoints(v) => format!("Action Points {v:+}"),
        Effect::Armor(v) => format!("Armor {v:+}"),
        Effect::Mana {
            dependent,
            increase_per_point,
        } => format!("Mana {increase_per_point:+}/point of {dependent}"),
        Effect::OnLvlUp(OnLvlUp::AddSkillPoints(v)) => {
            format!("{v:+} Skill Points per level")
        }
        Effect::OnLvlUp(OnLvlUp::AddAbilityPoints(v)) => {
            format!("{v:+} Ability Points per level")
        }
        Effect::OnLvlUp(OnLvlUp::AddCharacteristicPoints(v)) => {
            format!("{v:+} Characteristic Points per level")
        }
    }
}

fn format_ability_type(t: AbilityType) -> String {
    match t {
        AbilityType::Stance => "Stance",
        AbilityType::Attack => "Attack",
        AbilityType::Debuff => "Debuff",
        AbilityType::Peaceful => "Peaceful",
        AbilityType::Passive => "Passive",
        AbilityType::Touch => "Touch",
    }
    .to_string()
}

fn format_ability_check(check: &AbilityCheck) -> String {
    check.to_string()
}

fn render_right_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    icons: &UiIcons,
    character: &CharacterQueryDataItem,
    weapon_registry: &crate::network::ClientWeaponRegistry,
    equipment_registry: &crate::network::ClientEquipmentRegistry,
    item_registry: &crate::network::ClientItemRegistry,
    ui_events: &mut UiEvents,
) {
    let gap = height * 0.03 / 2.0;
    let wallet = character.wallet;

    let inventory_items: Vec<Option<InventoryTooltip>> = character
        .inventory
        .0
        .iter()
        .map(|inv_item| match inv_item {
            shared::InventoryItem::Weapon(name) => {
                weapon_registry
                    .0
                    .get(name)
                    .map(|w| InventoryTooltip::Weapon {
                        name: w.name.clone(),
                        kind: w.kind.to_string(),
                        attack: format!("{:+}", w.attack),
                        damage: w.damage.clone(),
                        range: w.range.to_string(),
                        condition: w.condition.clone().unwrap_or_default(),
                    })
            }
            shared::InventoryItem::Equipment(name) => {
                equipment_registry
                    .0
                    .get(name)
                    .map(|e| InventoryTooltip::Equipment {
                        name: e.name.clone(),
                        slot: e.slot.to_string(),
                        description: e.description.clone(),
                        effects: e.effects.iter().map(format_effect).collect(),
                    })
            }
            shared::InventoryItem::Item(name) => {
                item_registry.0.get(name).map(|i| InventoryTooltip::Item {
                    name: i.name.clone(),
                    description: i.description.clone(),
                })
            }
        })
        .collect();

    let equipped_items: Vec<Option<InventoryTooltip>> = character
        .equipment
        .0
        .values()
        .flat_map(|names| names.iter())
        .map(|name| {
            equipment_registry
                .0
                .get(name)
                .map(|e| InventoryTooltip::Equipment {
                    name: e.name.clone(),
                    slot: e.slot.to_string(),
                    description: e.description.clone(),
                    effects: e.effects.iter().map(format_effect).collect(),
                })
        })
        .collect();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let equipped_size = egui::vec2(width, height * 0.41);
        let (equipped_rect, _) = ui.allocate_exact_size(equipped_size, egui::Sense::hover());
        let mut equipped_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(equipped_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        if let Some(i) = EquippedGear::new(icons.inventory_placeholder.id())
            .items(equipped_items)
            .show(&mut equipped_ui)
        {
            ui_events.inventory.write(InventoryChanged::UnequipGear(i));
        }

        ui.add_space(gap);

        let wallet_size = egui::vec2(width, height * 0.08);
        let (wallet_rect, _) = ui.allocate_exact_size(wallet_size, egui::Sense::hover());
        let mut wallet_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(wallet_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let result = WalletWidget::new(
            wallet.gold(),
            wallet.silver(),
            wallet.copper(),
            icons.wallet_gold.id(),
            icons.wallet_silver.id(),
            icons.wallet_copper.id(),
        )
        .show(&mut wallet_ui);
        send_wallet_events(&mut ui_events.wallet, result);

        ui.add_space(gap);

        let inventory_size = egui::vec2(width, height * 0.48);
        let (inventory_rect, _) = ui.allocate_exact_size(inventory_size, egui::Sense::hover());
        let mut inventory_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inventory_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        match Inventory::new(icons.inventory_placeholder.id())
            .items(inventory_items)
            .show(&mut inventory_ui)
        {
            Some(CellAction::Primary(i)) => {
                ui_events.inventory.write(InventoryChanged::Equip(i));
            }
            Some(CellAction::Remove(i)) => {
                ui_events.inventory.write(InventoryChanged::Remove(i));
            }
            None => {}
        }
    });
}

fn send_wallet_events(events: &mut MessageWriter<WalletChanged>, result: WalletResponse) {
    for delta in [result.gold, result.silver, result.copper]
        .into_iter()
        .flatten()
    {
        events.write(WalletChanged(delta));
    }
}

fn apply_wallet_changes(
    mut query: Query<&mut Wallet, With<ActiveCharacter>>,
    mut reader: MessageReader<WalletChanged>,
) {
    let Ok(mut wallet) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        let new_val = wallet.0 as i64 + event.0;
        wallet.0 = new_val.max(0) as u64;
    }
}

const MAX_EQUIPPED_WEAPONS: usize = 3;

fn apply_inventory_changes(
    mut query: Query<
        (
            &mut InventoryComponent,
            &mut CharacterEquipment,
            &mut CharacterWeaponNames,
        ),
        With<ActiveCharacter>,
    >,
    mut reader: MessageReader<InventoryChanged>,
    equipment_registry: Res<crate::network::ClientEquipmentRegistry>,
) {
    let Ok((mut inventory, mut equipment, mut weapons)) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        match event {
            InventoryChanged::Equip(idx) => {
                let idx = *idx;
                if idx >= inventory.0.len() {
                    continue;
                }
                match &inventory.0[idx] {
                    InventoryItem::Equipment(name) => {
                        let name = name.clone();
                        if let Some(eq) = equipment_registry.0.get(&name) {
                            let slot = eq.slot;
                            if slot != EquipmentSlot::Ring {
                                if let Some(existing) = equipment.0.get(&slot) {
                                    for old_name in existing.clone() {
                                        inventory.0.push(InventoryItem::Equipment(old_name));
                                    }
                                }
                                equipment.0.insert(slot, vec![name]);
                            } else {
                                equipment.0.entry(slot).or_default().push(name);
                            }
                            inventory.0.remove(idx);
                        }
                    }
                    InventoryItem::Weapon(name) => {
                        if weapons.0.len() >= MAX_EQUIPPED_WEAPONS {
                            continue;
                        }
                        let name = name.clone();
                        weapons.0.push(name);
                        inventory.0.remove(idx);
                    }
                    InventoryItem::Item(_) => {}
                }
            }
            InventoryChanged::Remove(idx) => {
                let idx = *idx;
                if idx < inventory.0.len() {
                    inventory.0.remove(idx);
                }
            }
            InventoryChanged::UnequipGear(idx) => {
                let idx = *idx;
                let mut current = 0;
                let mut target: Option<(EquipmentSlot, usize)> = None;
                for (slot, names) in equipment.0.iter() {
                    if idx < current + names.len() {
                        target = Some((*slot, idx - current));
                        break;
                    }
                    current += names.len();
                }
                if let Some((slot, inner_idx)) = target {
                    let name = equipment.0.get_mut(&slot).unwrap().remove(inner_idx);
                    if equipment.0.get(&slot).is_some_and(|v| v.is_empty()) {
                        equipment.0.remove(&slot);
                    }
                    inventory.0.push(InventoryItem::Equipment(name));
                }
            }
            InventoryChanged::UnequipWeapon(idx) => {
                let idx = *idx;
                if idx < weapons.0.len() {
                    let name = weapons.0.remove(idx);
                    inventory.0.push(InventoryItem::Weapon(name));
                }
            }
            InventoryChanged::AddExisting(item) => {
                inventory.0.push(item.clone());
            }
        }
    }
}

fn apply_create_item(
    mut query: Query<&mut InventoryComponent, With<ActiveCharacter>>,
    mut reader: MessageReader<CreateItem>,
    mut weapon_registry: ResMut<crate::network::ClientWeaponRegistry>,
    mut equipment_registry: ResMut<crate::network::ClientEquipmentRegistry>,
    mut item_registry: ResMut<crate::network::ClientItemRegistry>,
    mut pending_messages: ResMut<crate::network::PendingClientMessages>,
) {
    let Ok(mut inventory) = query.single_mut() else {
        return;
    };
    for event in reader.read() {
        match event {
            CreateItem::Weapon(weapon) => {
                let item_name = weapon.name.clone();
                weapon_registry
                    .0
                    .weapons
                    .insert(item_name.clone(), weapon.clone());
                inventory.0.push(InventoryItem::Weapon(item_name));
                save_weapons_to_file(&weapon_registry.0);
                pending_messages
                    .0
                    .push(shared::ClientMessage::CreateWeapon {
                        weapon: weapon.clone(),
                    });
            }
            CreateItem::Equipment(eq) => {
                let item_name = eq.name.clone();
                equipment_registry
                    .0
                    .equipment
                    .insert(item_name.clone(), eq.clone());
                inventory.0.push(InventoryItem::Equipment(item_name));
                save_equipment_to_file(&equipment_registry.0);
                pending_messages
                    .0
                    .push(shared::ClientMessage::CreateEquipment {
                        equipment: eq.clone(),
                    });
            }
            CreateItem::Item(item) => {
                let item_name = item.name.clone();
                item_registry
                    .0
                    .items
                    .insert(item_name.clone(), item.clone());
                inventory.0.push(InventoryItem::Item(item_name));
                save_items_to_file(&item_registry.0);
                pending_messages
                    .0
                    .push(shared::ClientMessage::CreateItem { item: item.clone() });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_weapons_to_file(registry: &shared::WeaponRegistry) {
    let weapons: Vec<&shared::Weapon> = registry.weapons.values().collect();
    match serde_json::to_string_pretty(&weapons) {
        Ok(json) => {
            if let Err(e) = std::fs::write("data/weapons.json", json) {
                warn!("Failed to save weapons.json: {e}");
            }
        }
        Err(e) => warn!("Failed to serialize weapons: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn save_weapons_to_file(_registry: &shared::WeaponRegistry) {}

#[cfg(not(target_arch = "wasm32"))]
fn save_equipment_to_file(registry: &shared::EquipmentRegistry) {
    let equipment: Vec<&shared::Equipment> = registry.equipment.values().collect();
    match serde_json::to_string_pretty(&equipment) {
        Ok(json) => {
            if let Err(e) = std::fs::write("data/equipment.json", json) {
                warn!("Failed to save equipment.json: {e}");
            }
        }
        Err(e) => warn!("Failed to serialize equipment: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn save_equipment_to_file(_registry: &shared::EquipmentRegistry) {}

#[cfg(not(target_arch = "wasm32"))]
fn save_items_to_file(registry: &shared::ItemRegistry) {
    let items: Vec<&shared::Item> = registry.items.values().collect();
    match serde_json::to_string_pretty(&items) {
        Ok(json) => {
            if let Err(e) = std::fs::write("data/items.json", json) {
                warn!("Failed to save items.json: {e}");
            }
        }
        Err(e) => warn!("Failed to serialize items: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn save_items_to_file(_registry: &shared::ItemRegistry) {}

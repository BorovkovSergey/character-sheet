use bevy::prelude::*;
use bevy_egui::egui;
use strum::IntoEnumIterator;
use ui_widgets::colors::MAIN_COLOR;

use shared::character::OnLvlUp;
use shared::{
    CharacteristicKind, Effect, EquipmentSlot, MeleeKind, Protection, RangeKind, Resist,
    WeaponGrip, WeaponKind,
};

use crate::events::CreateItem;

#[derive(Resource, Default)]
pub struct CreateItemOpen(pub bool);

const ITEM_TYPE_LABELS: [&str; 3] = ["Item", "Equipment", "Weapon"];

const EFFECT_TYPE_LABELS: [&str; 9] = [
    "Resist",
    "Skill",
    "Protection",
    "Initiative",
    "Characteristic",
    "Action Points",
    "Armor",
    "Mana",
    "OnLvlUp",
];

/// Renders an egui ComboBox populated from an `EnumIter + Display` enum.
/// `selected` is the index into the iteration order; it is clamped in-place.
fn enum_combo<E: IntoEnumIterator + std::fmt::Display>(
    ui: &mut egui::Ui,
    id: &str,
    selected: &mut usize,
    width: f32,
) {
    let variants: Vec<String> = E::iter().map(|v| v.to_string()).collect();
    *selected = (*selected).min(variants.len().saturating_sub(1));
    egui::ComboBox::from_id_salt(id)
        .selected_text(&variants[*selected])
        .width(width)
        .show_ui(ui, |ui| {
            for (i, label) in variants.iter().enumerate() {
                ui.selectable_value(selected, i, label.as_str());
            }
        });
}

/// Returns the `idx`-th variant from an `EnumIter`, or `None` if out of bounds.
fn nth_variant<E: IntoEnumIterator>(idx: usize) -> Option<E> {
    E::iter().nth(idx)
}

#[derive(Clone, Default)]
struct CreateItemState {
    item_type: usize,
    name: String,
    description: String,
    slot_idx: usize,
    damage: String,
    attack: String,
    weapon_kind_idx: usize,
    range_subtype_idx: usize,
    melee_subtype_idx: usize,
    grip_idx: usize,
    range: String,
    condition: String,
    // Effects
    effects: Vec<Effect>,
    effect_type_idx: usize,
    effect_sub_idx: usize,
    effect_value: String,
    effect_skill_idx: usize,
    effect_mana_value: String,
}

pub fn render_create_item_popup(
    ctx: &egui::Context,
    create_item_open: &mut CreateItemOpen,
    create_item_events: &mut MessageWriter<CreateItem>,
    format_effect: &dyn Fn(&Effect) -> String,
    skill_names: &[String],
    existing_item_names: &std::collections::BTreeSet<String>,
    existing_equipment_names: &std::collections::BTreeSet<String>,
    existing_weapon_names: &std::collections::BTreeSet<String>,
) {
    let screen = ctx.content_rect();

    egui::Area::new(egui::Id::new("create_item_backdrop"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(120));
            if resp.clicked() {
                create_item_open.0 = false;
            }
        });

    let dialog_w = (screen.width() * 0.38).max(360.0);
    let dialog_pos = egui::pos2(
        screen.center().x - dialog_w / 2.0,
        screen.center().y - screen.height() * 0.65 / 2.0,
    );

    let state_id = egui::Id::new("create_item_state");
    let mut state: CreateItemState = ctx.data(|d| d.get_temp(state_id)).unwrap_or_default();
    state.item_type = state.item_type.min(2);
    state.effect_type_idx = state.effect_type_idx.min(EFFECT_TYPE_LABELS.len() - 1);

    egui::Area::new(egui::Id::new("create_item_dialog"))
        .order(egui::Order::Foreground)
        .fixed_pos(dialog_pos)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(MAIN_COLOR)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200)))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(dialog_w - 32.0);

                    ui.heading("Create Item");
                    ui.add_space(8.0);

                    // Type selector
                    ui.horizontal(|ui| {
                        for (i, label) in ITEM_TYPE_LABELS.iter().enumerate() {
                            if ui.selectable_label(state.item_type == i, *label).clicked() {
                                state.item_type = i;
                            }
                        }
                    });
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Common fields
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut state.name);
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_singleline(&mut state.description);
                    });

                    match state.item_type {
                        0 => {}
                        1 => {
                            // Equipment
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Slot:");
                                enum_combo::<EquipmentSlot>(
                                    ui,
                                    "eq_slot",
                                    &mut state.slot_idx,
                                    80.0,
                                );
                            });
                        }
                        2 => {
                            // Weapon
                            const WEAPON_KIND_LABELS: [&str; 4] =
                                ["Range", "Melee", "Shield", "Bard Instrument"];
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Kind:");
                                for (i, label) in WEAPON_KIND_LABELS.iter().enumerate() {
                                    ui.radio_value(&mut state.weapon_kind_idx, i, *label);
                                }
                            });

                            let is_combat = state.weapon_kind_idx <= 1;

                            if is_combat {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label("Damage:");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut state.damage)
                                            .desired_width(80.0),
                                    );
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label("Attack:");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut state.attack)
                                            .desired_width(60.0),
                                    );
                                });
                                state.attack.retain(|c| c.is_ascii_digit() || c == '-');
                            }

                            ui.add_space(4.0);
                            match state.weapon_kind_idx {
                                0 => {
                                    ui.horizontal(|ui| {
                                        ui.label("Subtype:");
                                        enum_combo::<RangeKind>(
                                            ui,
                                            "range_sub",
                                            &mut state.range_subtype_idx,
                                            80.0,
                                        );
                                    });
                                }
                                1 => {
                                    ui.horizontal(|ui| {
                                        ui.label("Subtype:");
                                        enum_combo::<MeleeKind>(
                                            ui,
                                            "melee_sub",
                                            &mut state.melee_subtype_idx,
                                            80.0,
                                        );
                                    });
                                }
                                _ => {}
                            }

                            if is_combat {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label("Grip:");
                                    enum_combo::<WeaponGrip>(
                                        ui,
                                        "wpn_grip",
                                        &mut state.grip_idx,
                                        100.0,
                                    );
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut state.range)
                                            .desired_width(60.0),
                                    );
                                });
                                state.range.retain(|c| c.is_ascii_digit());
                            }

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Condition:");
                                ui.text_edit_singleline(&mut state.condition);
                            });
                        }
                        _ => {}
                    }

                    // Effects editor (for Equipment and Weapon)
                    if state.item_type == 1 || state.item_type == 2 {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.label("Effects:");

                        let mut remove_idx = None;
                        for (i, effect) in state.effects.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format_effect(effect));
                                if ui.small_button("x").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        }
                        if let Some(i) = remove_idx {
                            state.effects.remove(i);
                        }

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("eff_type")
                                .selected_text(EFFECT_TYPE_LABELS[state.effect_type_idx])
                                .width(110.0)
                                .show_ui(ui, |ui| {
                                    for (i, label) in EFFECT_TYPE_LABELS.iter().enumerate() {
                                        ui.selectable_value(&mut state.effect_type_idx, i, *label);
                                    }
                                });

                            render_effect_fields(ui, &mut state, skill_names);

                            if ui.small_button("+").clicked() {
                                if let Some(effect) = build_effect_from_state(&state, skill_names) {
                                    state.effects.push(effect);
                                    state.effect_value.clear();
                                    state.effect_mana_value.clear();
                                }
                            }
                        });
                        state
                            .effect_value
                            .retain(|c| c.is_ascii_digit() || c == '-');
                        state
                            .effect_mana_value
                            .retain(|c| c.is_ascii_digit() || c == '-');
                    }

                    ui.add_space(12.0);

                    let trimmed_name = state.name.trim().to_string();
                    let name_taken = match state.item_type {
                        0 => existing_item_names.contains(&trimmed_name),
                        1 => existing_equipment_names.contains(&trimmed_name),
                        2 => existing_weapon_names.contains(&trimmed_name),
                        _ => false,
                    };
                    let can_create = !trimmed_name.is_empty() && !name_taken;
                    if ui
                        .add_enabled(can_create, egui::Button::new("Create"))
                        .clicked()
                    {
                        match state.item_type {
                            0 => {
                                create_item_events.write(CreateItem::Item(shared::Item {
                                    name: state.name.trim().to_string(),
                                    description: state.description.clone(),
                                }));
                            }
                            1 => {
                                let slot = nth_variant::<EquipmentSlot>(state.slot_idx)
                                    .unwrap_or(EquipmentSlot::Any);
                                create_item_events.write(CreateItem::Equipment(
                                    shared::Equipment {
                                        name: state.name.trim().to_string(),
                                        description: state.description.clone(),
                                        slot,
                                        effects: state.effects.clone(),
                                    },
                                ));
                            }
                            2 => {
                                let kind = match state.weapon_kind_idx {
                                    1 => {
                                        let sub = nth_variant::<MeleeKind>(state.melee_subtype_idx)
                                            .unwrap_or(MeleeKind::Slashing);
                                        WeaponKind::Melee(sub)
                                    }
                                    2 => WeaponKind::Shield,
                                    3 => WeaponKind::BardInstrument,
                                    _ => {
                                        let sub = nth_variant::<RangeKind>(state.range_subtype_idx)
                                            .unwrap_or(RangeKind::Bow);
                                        WeaponKind::Range(sub)
                                    }
                                };
                                let is_combat = state.weapon_kind_idx <= 1;
                                let grip = if is_combat {
                                    nth_variant::<WeaponGrip>(state.grip_idx)
                                        .unwrap_or(WeaponGrip::OneHanded)
                                } else {
                                    WeaponGrip::OneHanded
                                };
                                create_item_events.write(CreateItem::Weapon(shared::Weapon {
                                    name: state.name.trim().to_string(),
                                    description: state.description.clone(),
                                    damage: if is_combat {
                                        state.damage.clone()
                                    } else {
                                        String::new()
                                    },
                                    attack: if is_combat {
                                        state.attack.parse().unwrap_or(0)
                                    } else {
                                        0
                                    },
                                    kind,
                                    grip,
                                    range: if is_combat {
                                        state.range.parse().unwrap_or(1)
                                    } else {
                                        0
                                    },
                                    effects: state.effects.clone(),
                                    condition: if state.condition.trim().is_empty() {
                                        None
                                    } else {
                                        Some(state.condition.trim().to_string())
                                    },
                                }));
                            }
                            _ => {}
                        }
                        state = CreateItemState::default();
                        create_item_open.0 = false;
                    }
                });
        });

    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

fn render_effect_fields(ui: &mut egui::Ui, state: &mut CreateItemState, skill_names: &[String]) {
    match state.effect_type_idx {
        0 => {
            enum_combo::<Resist>(ui, "eff_sub", &mut state.effect_sub_idx, 80.0);
            ui.add(egui::TextEdit::singleline(&mut state.effect_value).desired_width(40.0));
        }
        1 => {
            if !skill_names.is_empty() {
                state.effect_skill_idx = state.effect_skill_idx.min(skill_names.len() - 1);
                egui::ComboBox::from_id_salt("eff_skill")
                    .selected_text(&skill_names[state.effect_skill_idx])
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for (i, name) in skill_names.iter().enumerate() {
                            ui.selectable_value(&mut state.effect_skill_idx, i, name.as_str());
                        }
                    });
            }
            ui.add(egui::TextEdit::singleline(&mut state.effect_value).desired_width(40.0));
        }
        2 => {
            enum_combo::<Protection>(ui, "eff_sub", &mut state.effect_sub_idx, 80.0);
            ui.add(egui::TextEdit::singleline(&mut state.effect_value).desired_width(40.0));
        }
        3 | 5 | 6 => {
            ui.add(egui::TextEdit::singleline(&mut state.effect_value).desired_width(40.0));
        }
        4 => {
            enum_combo::<CharacteristicKind>(ui, "eff_sub", &mut state.effect_sub_idx, 60.0);
            ui.add(egui::TextEdit::singleline(&mut state.effect_value).desired_width(40.0));
        }
        7 => {
            enum_combo::<CharacteristicKind>(ui, "eff_sub", &mut state.effect_sub_idx, 60.0);
            ui.add(
                egui::TextEdit::singleline(&mut state.effect_mana_value)
                    .desired_width(40.0)
                    .hint_text("+/pt"),
            );
        }
        8 => {
            enum_combo::<OnLvlUp>(ui, "eff_sub", &mut state.effect_sub_idx, 130.0);
            ui.add(egui::TextEdit::singleline(&mut state.effect_value).desired_width(40.0));
        }
        _ => {}
    }
}

fn build_effect_from_state(state: &CreateItemState, skill_names: &[String]) -> Option<Effect> {
    match state.effect_type_idx {
        0 => {
            let val: i32 = state.effect_value.parse().ok()?;
            let resist = nth_variant::<Resist>(state.effect_sub_idx)?;
            Some(Effect::Resist(resist, val))
        }
        1 => {
            let val: i32 = state.effect_value.parse().ok()?;
            let name = skill_names.get(state.effect_skill_idx)?;
            Some(Effect::Skill(name.clone(), val))
        }
        2 => {
            let val: i32 = state.effect_value.parse().ok()?;
            let prot = nth_variant::<Protection>(state.effect_sub_idx)?;
            Some(Effect::Protection(prot, val))
        }
        3 => {
            let val: i32 = state.effect_value.parse().ok()?;
            Some(Effect::Initiative(val))
        }
        4 => {
            let val: i32 = state.effect_value.parse().ok()?;
            let kind = nth_variant::<CharacteristicKind>(state.effect_sub_idx)?;
            Some(Effect::Characteristic(kind, val))
        }
        5 => {
            let val: i32 = state.effect_value.parse().ok()?;
            Some(Effect::ActionPoints(val))
        }
        6 => {
            let val: i32 = state.effect_value.parse().ok()?;
            Some(Effect::Armor(val))
        }
        7 => {
            let val: i32 = state.effect_mana_value.parse().ok()?;
            let dependent = nth_variant::<CharacteristicKind>(state.effect_sub_idx)?;
            Some(Effect::Mana {
                dependent,
                increase_per_point: val,
            })
        }
        8 => {
            let val: i32 = state.effect_value.parse().ok()?;
            let variant = nth_variant::<OnLvlUp>(state.effect_sub_idx)?;
            let on_lvl_up = match variant {
                OnLvlUp::AddSkillPoints(_) => OnLvlUp::AddSkillPoints(val),
                OnLvlUp::AddAbilityPoints(_) => OnLvlUp::AddAbilityPoints(val),
                OnLvlUp::AddCharacteristicPoints(_) => OnLvlUp::AddCharacteristicPoints(val),
            };
            Some(Effect::OnLvlUp(on_lvl_up))
        }
        _ => None,
    }
}

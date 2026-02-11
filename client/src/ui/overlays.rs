use bevy::prelude::*;
use bevy_egui::egui;
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR};
use ui_widgets::molecules::{AbilityCard, SmallAbility};

use shared::CharacterTrait;

use crate::events::{LearnAbility, LearnTrait};

use super::helpers::{check_trait_requirement, format_effect};
use super::icons::UiIcons;
use super::layout::CharacterQueryDataItem;
use super::params::{LearnAbilityOpen, LearnTraitOpen, Registries, UiEvents};

pub(super) fn render_learn_ability_overlay(
    ctx: &egui::Context,
    character: &CharacterQueryDataItem,
    icons: &UiIcons,
    registries: &Registries,
    ui_events: &mut UiEvents,
    learn_ability_open: &mut ResMut<LearnAbilityOpen>,
) {
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
            // grid[row][col] -- row maps directly to LearnScreenPosition.row,
            // col maps to LearnScreenPosition.column.
            // Tuple: (name, mp_cost, can_learn, already_learned)
            let mut grid: [[Option<(&str, Option<u32>, bool, bool)>; 3]; 3] = Default::default();
            if let Some(class_abilities) = registries.abilities.get_class_abilities(character.class)
            {
                let known: &[String] = character.ability_names;
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
            let class_abilities = registries.abilities.get_class_abilities(character.class);
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
                                                ability.requirements.as_ref().and_then(|r| r.mp),
                                            )
                                            .ap_cost(
                                                ability
                                                    .requirements
                                                    .as_ref()
                                                    .and_then(|r| r.action_points),
                                            )
                                            .self_only(ability.self_only)
                                            .range(
                                                ability.requirements.as_ref().and_then(|r| r.range),
                                            )
                                            .ability_type(ability.ability_type.to_string())
                                            .check(
                                                ability
                                                    .check
                                                    .as_ref()
                                                    .map(|c| c.to_string())
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
                                            egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
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

/// State stored in egui temp data for the "Learn Trait" dialog.
#[derive(Clone, Default)]
struct LearnTraitDialogState {
    selected: Option<String>,
}

pub(super) fn render_learn_trait_overlay(
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

            let known_traits: &[String] = character.trait_names;

            // Build sorted list of all traits
            let mut all_traits: Vec<(&String, &CharacterTrait)> =
                trait_registry.traits.iter().collect();
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
                            check_trait_requirement(character.stats, ct.condition.as_ref());
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

            let can_confirm = state
                .selected
                .as_ref()
                .is_some_and(|name| character.trait_pts.0 > 0 && !known_traits.contains(name));
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

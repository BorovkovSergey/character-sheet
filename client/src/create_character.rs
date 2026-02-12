use bevy::prelude::*;
use bevy_egui::egui;
use strum::IntoEnumIterator;
use ui_widgets::colors::{MAIN_COLOR, STROKE_COLOR, TEXT_COLOR};
use ui_widgets::composites::{
    Characteristics, GridAction, Points, SkillEntry, Skills, TraitEntry, Traits,
};

use shared::{CharacterSkill, Characteristics as Stats, Class, Race};

use crate::ui::{format_effect, render_trait_select_overlay, TraitSelectMode};

use crate::network::{ClientSkillRegistry, ClientTraitRegistry};
use crate::portrait::{PendingCreationPortrait, PortraitPickerResult};

#[derive(Resource, Default)]
pub struct CreateCharacterOpen(pub bool);

#[derive(Clone)]
struct CreateCharacterState {
    name: String,
    race_idx: usize,
    class_idx: usize,
    characteristic_points: u32,
    skill_points: i32,
    stats: Stats,
    skills: Vec<CharacterSkill>,
    selected_traits: Vec<String>,
    traits_open: bool,
    portrait_bytes: Option<Vec<u8>>,
    portrait_texture: Option<egui::TextureHandle>,
}

impl Default for CreateCharacterState {
    fn default() -> Self {
        let intellect = 1;
        Self {
            name: String::new(),
            race_idx: 0,
            class_idx: 0,
            characteristic_points: 18,
            skill_points: 10 + intellect as i32,
            stats: Stats {
                strength: shared::Characteristic::new(1),
                dexterity: shared::Characteristic::new(1),
                endurance: shared::Characteristic::new(1),
                perception: shared::Characteristic::new(1),
                magic: shared::Characteristic::new(1),
                willpower: shared::Characteristic::new(1),
                intellect: shared::Characteristic::new(intellect),
                charisma: shared::Characteristic::new(1),
            },
            skills: Vec::new(),
            selected_traits: Vec::new(),
            traits_open: false,
            portrait_bytes: None,
            portrait_texture: None,
        }
    }
}

pub fn render_create_character_overlay(
    ctx: &egui::Context,
    create_open: &mut CreateCharacterOpen,
    skill_registry: &ClientSkillRegistry,
    trait_registry: &ClientTraitRegistry,
    pending_messages: &mut crate::network::PendingClientMessages,
    portrait_picker: &PortraitPickerResult,
    pending_creation_portrait: &mut PendingCreationPortrait,
) {
    let screen = ctx.content_rect();
    let state_id = egui::Id::new("create_character_state");

    // Semi-transparent backdrop
    egui::Area::new(egui::Id::new("create_character_backdrop"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(120));
            if resp.clicked() {
                create_open.0 = false;
                ctx.data_mut(|d| d.remove::<CreateCharacterState>(state_id));
            }
        });

    // Centered white dialog
    let dialog_w = (screen.width() * 0.5).max(440.0).min(700.0);

    let mut state: CreateCharacterState = ctx.data(|d| d.get_temp(state_id)).unwrap_or_default();

    // Poll portrait picker for newly selected file
    if let Ok(mut guard) = portrait_picker.0.lock() {
        if let Some(raw_bytes) = guard.take() {
            if let Some(png_bytes) = crate::portrait::process_raw_image(&raw_bytes) {
                if let Some(texture) =
                    crate::portrait::png_to_texture(ctx, "create_portrait_preview", &png_bytes)
                {
                    state.portrait_texture = Some(texture);
                    state.portrait_bytes = Some(png_bytes);
                }
            }
        }
    }

    let selected_class = Class::iter().nth(state.class_idx).unwrap_or_default();

    egui::Window::new("Create Character")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .order(egui::Order::Foreground)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(MAIN_COLOR)
                .corner_radius(12.0)
                .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                .inner_margin(egui::Margin::same(20)),
        )
        .min_width(dialog_w)
        .max_width(dialog_w)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Create Character")
                        .size(24.0)
                        .color(TEXT_COLOR)
                        .strong(),
                );
                ui.add_space(12.0);
            });

            ui.separator();
            ui.add_space(12.0);

            // Portrait upload
            ui.horizontal(|ui| {
                let preview_size = 64.0;
                if let Some(texture) = &state.portrait_texture {
                    let img = egui::Image::new(egui::load::SizedTexture::new(
                        texture.id(),
                        egui::vec2(preview_size, preview_size),
                    ));
                    ui.add(img);
                } else {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(preview_size, preview_size),
                        egui::Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(rect, 8.0, egui::Color32::from_gray(40));
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "?",
                        egui::FontId::proportional(24.0),
                        TEXT_COLOR,
                    );
                }
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    let label = if state.portrait_bytes.is_some() {
                        "Change Portrait"
                    } else {
                        "Upload Portrait"
                    };
                    let button =
                        egui::Button::new(egui::RichText::new(label).size(14.0).color(TEXT_COLOR))
                            .corner_radius(6.0)
                            .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                            .fill(MAIN_COLOR);
                    if ui.add(button).clicked() {
                        crate::portrait::spawn_portrait_picker(portrait_picker);
                    }
                });
            });
            ui.add_space(12.0);

            // Name
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Name:").size(15.0).color(TEXT_COLOR));
                ui.add_sized(
                    [ui.available_width(), 24.0],
                    egui::TextEdit::singleline(&mut state.name),
                );
            });
            ui.add_space(10.0);

            // Race & Class
            let races: Vec<String> = Race::iter().map(|r| r.to_string()).collect();
            state.race_idx = state.race_idx.min(races.len().saturating_sub(1));
            let classes: Vec<String> = Class::iter().map(|c| c.to_string()).collect();
            state.class_idx = state.class_idx.min(classes.len().saturating_sub(1));
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Race:").size(15.0).color(TEXT_COLOR));
                egui::ComboBox::from_id_salt("create_char_race")
                    .selected_text(&races[state.race_idx])
                    .show_ui(ui, |ui| {
                        for (i, label) in races.iter().enumerate() {
                            ui.selectable_value(&mut state.race_idx, i, label.as_str());
                        }
                    });
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Class:").size(15.0).color(TEXT_COLOR));
                egui::ComboBox::from_id_salt("create_char_class")
                    .selected_text(&classes[state.class_idx])
                    .show_ui(ui, |ui| {
                        for (i, label) in classes.iter().enumerate() {
                            ui.selectable_value(&mut state.class_idx, i, label.as_str());
                        }
                    });
                ui.add_space(12.0);
                let trait_label = format!("Traits ({}/3)", state.selected_traits.len());
                let trait_button = egui::Button::new(
                    egui::RichText::new(trait_label)
                        .size(14.0)
                        .color(TEXT_COLOR),
                )
                .corner_radius(6.0)
                .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                .fill(MAIN_COLOR);
                if ui.add(trait_button).clicked() {
                    state.traits_open = !state.traits_open;
                }
            });

            ui.add_space(12.0);

            // Points (editable)
            let width = ui.available_width();
            let points_size = egui::vec2(width, 36.0);
            let (points_rect, _) = ui.allocate_exact_size(points_size, egui::Sense::hover());
            let mut points_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(points_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            let pts = Points::new(state.characteristic_points, state.skill_points)
                .editable(true)
                .show(&mut points_ui);
            state.characteristic_points = pts.characteristic_points;
            state.skill_points = pts.skill_points;
            ui.add_space(8.0);

            // Characteristics
            let char_values: Vec<(String, u32)> = [
                ("STR", state.stats.strength.level),
                ("DEX", state.stats.dexterity.level),
                ("END", state.stats.endurance.level),
                ("PER", state.stats.perception.level),
                ("MAG", state.stats.magic.level),
                ("WIL", state.stats.willpower.level),
                ("INT", state.stats.intellect.level),
                ("CHA", state.stats.charisma.level),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

            let char_size = egui::vec2(width, 80.0);
            let (char_rect, _) = ui.allocate_exact_size(char_size, egui::Sense::hover());
            let mut char_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(char_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            let prev_intellect = state.stats.intellect.level;
            match Characteristics::new(char_values)
                .edit_mode(true, state.characteristic_points)
                .show(&mut char_ui)
            {
                Some(GridAction::Upgrade(idx)) => {
                    let stat = match idx {
                        0 => &mut state.stats.strength,
                        1 => &mut state.stats.dexterity,
                        2 => &mut state.stats.endurance,
                        3 => &mut state.stats.perception,
                        4 => &mut state.stats.magic,
                        5 => &mut state.stats.willpower,
                        6 => &mut state.stats.intellect,
                        7 => &mut state.stats.charisma,
                        _ => unreachable!(),
                    };
                    let spent = stat.up(state.characteristic_points);
                    state.characteristic_points -= spent;
                }
                Some(GridAction::Downgrade(idx)) => {
                    let stat = match idx {
                        0 => &mut state.stats.strength,
                        1 => &mut state.stats.dexterity,
                        2 => &mut state.stats.endurance,
                        3 => &mut state.stats.perception,
                        4 => &mut state.stats.magic,
                        5 => &mut state.stats.willpower,
                        6 => &mut state.stats.intellect,
                        7 => &mut state.stats.charisma,
                        _ => unreachable!(),
                    };
                    if stat.level > 1 {
                        let refund = stat.level;
                        stat.level -= 1;
                        state.characteristic_points += refund;
                    }
                }
                None => {}
            }
            // Intellect change adjusts skill points
            let new_intellect = state.stats.intellect.level;
            if new_intellect != prev_intellect {
                state.skill_points += new_intellect as i32 - prev_intellect as i32;
            }

            ui.add_space(8.0);

            // Skills (depend on selected class)
            let skill_entries: Vec<SkillEntry> = skill_registry
                .get_class_skills(&selected_class)
                .into_iter()
                .flat_map(|skills| skills.iter())
                .map(|(name, skill)| {
                    let level = state
                        .skills
                        .iter()
                        .find(|s| s.name == *name)
                        .map_or(0, |s| s.level as i32);
                    let max_level = state.stats.get_level(skill.dependency);
                    SkillEntry {
                        name: name.clone(),
                        dependency: skill.dependency.to_string(),
                        level,
                        max_level,
                    }
                })
                .collect();

            let skill_size = egui::vec2(width, 160.0);
            let (skill_rect, _) = ui.allocate_exact_size(skill_size, egui::Sense::hover());
            let mut skill_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(skill_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            match Skills::new(skill_entries)
                .edit_mode(true, state.skill_points.max(0) as u32)
                .show(&mut skill_ui)
            {
                Some(GridAction::Upgrade(idx)) => {
                    if let Some(skill_name) = skill_registry
                        .get_class_skills(&selected_class)
                        .into_iter()
                        .flat_map(|skills| skills.keys())
                        .nth(idx)
                    {
                        let skill_def = skill_registry.get_skill(&selected_class, skill_name);
                        let max_level = skill_def
                            .map(|s| state.stats.get_level(s.dependency))
                            .unwrap_or(0);

                        if let Some(cs) = state.skills.iter_mut().find(|s| s.name == *skill_name) {
                            let spent = cs.up(state.skill_points.max(0) as u32, max_level);
                            state.skill_points -= spent as i32;
                        } else if state.skill_points >= 1 && max_level >= 1 {
                            state.skills.push(CharacterSkill::new(skill_name.clone()));
                            state.skill_points -= 1;
                        }
                    }
                }
                Some(GridAction::Downgrade(idx)) => {
                    if let Some(skill_name) = skill_registry
                        .get_class_skills(&selected_class)
                        .into_iter()
                        .flat_map(|skills| skills.keys())
                        .nth(idx)
                    {
                        if let Some(cs) = state.skills.iter_mut().find(|s| s.name == *skill_name) {
                            if cs.level > 0 {
                                let refund = cs.level;
                                cs.level -= 1;
                                state.skill_points += refund as i32;
                            }
                        }
                    }
                    // Remove skills with level 0
                    state.skills.retain(|s| s.level > 0);
                }
                None => {}
            }

            ui.add_space(8.0);

            // Selected traits
            let trait_entries: Vec<TraitEntry> = state
                .selected_traits
                .iter()
                .filter_map(|name| {
                    trait_registry.get(name).map(|ct| TraitEntry {
                        name: name.clone(),
                        description: ct.description.clone(),
                        effects: ct.effects.iter().map(format_effect).collect(),
                    })
                })
                .collect();
            let traits_size = egui::vec2(width, 80.0);
            let (traits_rect, _) = ui.allocate_exact_size(traits_size, egui::Sense::hover());
            let mut traits_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(traits_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            traits_ui.add_sized(
                [traits_rect.width(), traits_rect.height()],
                Traits::new(trait_entries),
            );

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.vertical_centered(|ui| {
                let has_over_limit_skills = skill_registry
                    .get_class_skills(&selected_class)
                    .into_iter()
                    .flat_map(|skills| skills.iter())
                    .any(|(name, skill)| {
                        let level = state
                            .skills
                            .iter()
                            .find(|s| s.name == *name)
                            .map_or(0, |s| s.level);
                        level > 0 && level > state.stats.get_level(skill.dependency)
                    });
                let all_points_spent = state.characteristic_points == 0 && state.skill_points == 0;
                let can_create = !state.name.trim().is_empty()
                    && !has_over_limit_skills
                    && all_points_spent
                    && state.selected_traits.len() == 3;
                let button =
                    egui::Button::new(egui::RichText::new("Create").size(16.0).color(TEXT_COLOR))
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                        .fill(MAIN_COLOR)
                        .min_size(egui::vec2(dialog_w * 0.5, 36.0));

                if ui.add_enabled(can_create, button).clicked() {
                    let selected_race = Race::iter().nth(state.race_idx).unwrap_or_default();
                    let selected_class = Class::iter().nth(state.class_idx).unwrap_or_default();
                    // Store portrait bytes for upload after CharacterCreated response
                    pending_creation_portrait.0 = state.portrait_bytes.take();
                    pending_messages
                        .0
                        .push(shared::ClientMessage::CreateCharacter {
                            name: state.name.clone(),
                            race: selected_race,
                            class: selected_class,
                            stats: state.stats.clone(),
                            skills: state.skills.clone(),
                            traits: state.selected_traits.clone(),
                        });
                    state = CreateCharacterState::default();
                    create_open.0 = false;
                }
            });
            ui.add_space(4.0);
        });

    // Trait selection overlay
    if state.traits_open {
        let _ = render_trait_select_overlay(
            ctx,
            &state.stats,
            trait_registry,
            &mut state.selected_traits,
            &mut state.traits_open,
            TraitSelectMode::Multi { max_count: 3 },
            "create_trait",
        );
    }

    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

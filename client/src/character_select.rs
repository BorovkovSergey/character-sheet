use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use shared::{Character, Class, Race, Resource};
use ui_widgets::colors::{MAIN_COLOR, SECONDARY_COLOR, STROKE_COLOR, TEXT_COLOR};

/// Tracks which screen the app is currently displaying.
#[derive(Debug, Clone, Resource)]
pub enum AppScreen {
    /// The character selection menu is visible.
    CharacterSelect,
    /// The character sheet is visible for the selected character.
    CharacterSheet(Character),
}

impl Default for AppScreen {
    fn default() -> Self {
        Self::CharacterSelect
    }
}

/// Holds the list of available characters for selection.
#[derive(Debug, Clone, Resource)]
pub struct CharacterList {
    pub characters: Vec<Character>,
}

impl Default for CharacterList {
    fn default() -> Self {
        Self {
            characters: create_mock_characters(),
        }
    }
}

fn create_mock_characters() -> Vec<Character> {
    vec![
        Character {
            name: "Eldrin Shadowmere".to_string(),
            race: Race::DarkHalfElf,
            class: Class::Bard,
            level: 5,
            experience: 6400,
            hp: Resource::new(35),
            mana: Resource::new(20),
            action_points: Resource::new(4),
            ..Character::new("".to_string())
        },
        Character {
            name: "Thalia Nightwhisper".to_string(),
            race: Race::DarkHalfElf,
            class: Class::Bard,
            level: 3,
            experience: 2700,
            hp: Resource::new(24),
            mana: Resource::new(15),
            action_points: Resource::new(3),
            ..Character::new("".to_string())
        },
        Character {
            name: "Grimjaw the Unyielding".to_string(),
            race: Race::DarkHalfElf,
            class: Class::Bard,
            level: 8,
            experience: 34000,
            hp: Resource::new(52),
            mana: Resource::new(30),
            action_points: Resource::new(5),
            ..Character::new("".to_string())
        },
        Character {
            name: "Seraphina Duskwalker".to_string(),
            race: Race::DarkHalfElf,
            class: Class::Bard,
            level: 1,
            experience: 0,
            hp: Resource::new(20),
            mana: Resource::new(10),
            action_points: Resource::new(3),
            ..Character::new("".to_string())
        },
    ]
}

pub struct CharacterSelectPlugin;

impl Plugin for CharacterSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppScreen>()
            .init_resource::<CharacterList>()
            .add_systems(EguiPrimaryContextPass, render_character_select);
    }
}

fn render_character_select(
    mut contexts: EguiContexts,
    mut app_screen: ResMut<AppScreen>,
    character_list: Res<CharacterList>,
) -> Result {
    if !matches!(*app_screen, AppScreen::CharacterSelect) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |_ui| {});

    let screen_rect = ctx.viewport_rect();
    let panel_width = (screen_rect.width() * 0.4).max(340.0).min(500.0);
    let panel_max_height = screen_rect.height() * 0.7;

    egui::Area::new(egui::Id::new("character_select_menu"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(SECONDARY_COLOR)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0, STROKE_COLOR))
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.set_width(panel_width);

                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Select Character")
                                .size(24.0)
                                .color(TEXT_COLOR)
                                .strong(),
                        );
                        ui.add_space(12.0);
                    });

                    ui.separator();
                    ui.add_space(8.0);

                    let mut selected: Option<Character> = None;

                    egui::ScrollArea::vertical()
                        .max_height(panel_max_height)
                        .show(ui, |ui| {
                            for character in &character_list.characters {
                                let response = render_character_entry(ui, character, panel_width);
                                if response.clicked() {
                                    selected = Some(character.clone());
                                }
                                ui.add_space(6.0);
                            }
                        });

                    if let Some(character) = selected {
                        *app_screen = AppScreen::CharacterSheet(character);
                    }
                });
        });

    Ok(())
}

/// Renders a single character entry as a clickable card.
fn render_character_entry(
    ui: &mut egui::Ui,
    character: &Character,
    panel_width: f32,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(panel_width, 64.0), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let bg_color = if response.hovered() {
            MAIN_COLOR
        } else {
            SECONDARY_COLOR
        };

        painter.rect_filled(rect, 6.0, bg_color);
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, STROKE_COLOR),
            egui::StrokeKind::Inside,
        );

        let text_left = rect.left() + 14.0;
        let name_pos = egui::pos2(text_left, rect.top() + 10.0);
        painter.text(
            name_pos,
            egui::Align2::LEFT_TOP,
            &character.name,
            egui::FontId::proportional(16.0),
            TEXT_COLOR,
        );

        let detail_text = format!(
            "{}  |  {}  |  Level {}",
            character.race, character.class, character.level
        );
        let detail_pos = egui::pos2(text_left, rect.top() + 34.0);
        painter.text(
            detail_pos,
            egui::Align2::LEFT_TOP,
            detail_text,
            egui::FontId::proportional(13.0),
            egui::Color32::from_rgb(0x88, 0x88, 0x99),
        );
    }

    response
}

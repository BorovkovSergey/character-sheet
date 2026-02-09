use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use ui_widgets::colors::MAIN_COLOR;
use ui_widgets::composites::{
    Abilities, Characteristics, EquippedGear, IdentityBar, Inventory, Points, Portrait, Skills,
    Stats, StatusBar, StatusBarResponse, Traits, Wallet,
};

use crate::components::{
    ActionPoints, ActiveCharacter, ActiveEffects, CharacterClass, CharacterName, CharacterRace,
    CharacterStats, CharacteristicPoints, Experience, Hp, Level, Mana, SkillPoints,
};
use crate::events::ResourceChanged;
use crate::state::AppScreen;

#[derive(Resource)]
struct UiIcons {
    heart: egui::TextureHandle,
    avatar_border_1: egui::TextureHandle,
    avatar_border_2: egui::TextureHandle,
    avatar_placeholder: egui::TextureHandle,
}

fn load_png_texture(ctx: &egui::Context, name: &str, png_bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(png_bytes).expect("failed to decode PNG");
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

fn load_svg_texture(
    ctx: &egui::Context,
    name: &str,
    svg_bytes: &[u8],
    scale: f32,
) -> egui::TextureHandle {
    let tree = resvg::usvg::Tree::from_data(svg_bytes, &resvg::usvg::Options::default())
        .expect("failed to parse SVG");
    let size = tree.size();
    let w = (size.width() * scale).ceil() as u32;
    let h = (size.height() * scale).ceil() as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).expect("failed to create pixmap");
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], pixmap.data());
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ResourceChanged>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    init_icons.run_if(not(resource_exists::<UiIcons>)),
                    render_ui.run_if(in_state(AppScreen::CharacterSheet)),
                ),
            )
            .add_systems(Update, apply_resource_changes);
    }
}

fn init_icons(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    commands.insert_resource(UiIcons {
        heart: load_png_texture(ctx, "heart", include_bytes!("../assets/heart.png")),
        avatar_border_1: load_svg_texture(
            ctx,
            "avatar_border_1",
            include_bytes!("../assets/avatar_border_1.svg"),
            4.0,
        ),
        avatar_border_2: load_svg_texture(
            ctx,
            "avatar_border_2",
            include_bytes!("../assets/avatar_border_2.svg"),
            4.0,
        ),
        avatar_placeholder: load_png_texture(
            ctx,
            "avatar_placeholder",
            include_bytes!("../assets/avatar_placeholder.png"),
        ),
    });
    Ok(())
}

const MARGIN: f32 = 0.02;
const COL_GAP: f32 = 0.01;
const COL1_WIDTH: f32 = 0.24;
const COL2_WIDTH: f32 = 0.46;
const COL3_WIDTH: f32 = 0.24;

#[allow(clippy::type_complexity)]
fn render_ui(
    mut contexts: EguiContexts,
    icons: Option<Res<UiIcons>>,
    character_query: Query<
        (
            &CharacterName,
            &CharacterRace,
            &CharacterClass,
            &Level,
            &Experience,
            &Hp,
            &Mana,
            &ActionPoints,
            &CharacterStats,
            &CharacteristicPoints,
            &SkillPoints,
            &ActiveEffects,
        ),
        With<ActiveCharacter>,
    >,
    mut events: MessageWriter<ResourceChanged>,
) -> Result {
    let Some(icons) = icons else {
        return Ok(());
    };

    let Ok((name, race, class, level, exp, hp, mana, ap, stats, char_pts, skill_pts, effects)) =
        character_query.single()
    else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let heart_icon = icons.heart.id();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(MAIN_COLOR))
        .show(ctx, |ui| {
            let total_w = ui.available_width();
            let total_h = ui.available_height();

            let margin = total_w * MARGIN;
            let gap = total_w * COL_GAP;
            let top_margin = margin / 2.0;
            let col_h = total_h - top_margin;

            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.add_space(top_margin);

            ui.horizontal(|ui| {
                ui.add_space(margin);
                render_left_column(
                    ui,
                    total_w * COL1_WIDTH,
                    col_h,
                    heart_icon,
                    icons.avatar_border_1.id(),
                    icons.avatar_border_2.id(),
                    icons.avatar_placeholder.id(),
                    &mut events,
                    name,
                    race,
                    class,
                    level,
                    exp,
                    hp,
                    mana,
                    ap,
                    effects,
                    stats,
                );
                ui.add_space(gap);
                render_center_column(ui, total_w * COL2_WIDTH, col_h, stats, char_pts, skill_pts);
                ui.add_space(gap);
                render_right_column(ui, total_w * COL3_WIDTH, col_h);
            });
        });

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_left_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    heart_icon: egui::TextureId,
    avatar_border_1: egui::TextureId,
    avatar_border_2: egui::TextureId,
    avatar_placeholder: egui::TextureId,
    events: &mut MessageWriter<ResourceChanged>,
    name: &CharacterName,
    race: &CharacterRace,
    class: &CharacterClass,
    level: &Level,
    exp: &Experience,
    hp: &Hp,
    mana: &Mana,
    ap: &ActionPoints,
    effects: &ActiveEffects,
    stats: &CharacterStats,
) {
    let gap = height * 0.03 / 4.0;
    let initiative = stats.0.perception.level as i32 + effects.initiative_bonus();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        ui.add_sized(
            [width, height * 0.30],
            Portrait::new(
                avatar_border_1,
                avatar_border_2,
                avatar_placeholder,
                level.0,
                exp.0,
            ),
        );
        ui.add_space(gap);
        ui.add_sized(
            [width, height * 0.11],
            IdentityBar::new(&name.0, &race.0.to_string(), &class.0.to_string()),
        );
        ui.add_space(gap);

        send_status_bar_events(ui, width, height * 0.16, hp, mana, ap, initiative, events);
        ui.add_space(gap);

        let resists = effects
            .get_resists()
            .into_iter()
            .map(|(r, v)| (r.to_string(), v))
            .collect();
        let protections = effects
            .get_protections()
            .into_iter()
            .map(|(p, v)| (p.to_string(), v))
            .collect();
        ui.add_sized(
            [width, height * 0.20],
            Stats::new(heart_icon, resists, protections),
        );
        ui.add_space(gap);

        send_status_bar_events(ui, width, height * 0.20, hp, mana, ap, initiative, events);
    });
}

#[allow(clippy::too_many_arguments)]
fn send_status_bar_events(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    hp: &Hp,
    mana: &Mana,
    ap: &ActionPoints,
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
        hp.current,
        hp.max,
        mana.current,
        mana.max,
        ap.current,
        ap.max,
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
            ResourceChanged::Hp(v) => hp.current = *v,
            ResourceChanged::Mp(v) => mana.current = *v,
            ResourceChanged::Ap(v) => ap.current = *v,
        }
    }
}

fn render_center_column(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    stats: &CharacterStats,
    char_pts: &CharacteristicPoints,
    skill_pts: &SkillPoints,
) {
    let gap = height * 0.03 / 4.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let characteristics = [
            ("STR", stats.0.strength.level),
            ("DEX", stats.0.dexterity.level),
            ("END", stats.0.endurance.level),
            ("PER", stats.0.perception.level),
            ("MAG", stats.0.magic.level),
            ("WIL", stats.0.willpower.level),
            ("INT", stats.0.intellect.level),
            ("CHA", stats.0.charisma.level),
        ];
        let char_values = characteristics
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        ui.add_sized([width, height * 0.14], Characteristics::new(char_values));
        ui.add_space(gap);
        ui.add_sized([width, height * 0.05], Points::new(char_pts.0, skill_pts.0));
        ui.add_space(gap);
        ui.add_sized([width, height * 0.24], Skills::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.14], Traits::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.40], Abilities::new());
    });
}

fn render_right_column(ui: &mut egui::Ui, width: f32, height: f32) {
    let gap = height * 0.03 / 2.0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        ui.add_sized([width, height * 0.41], EquippedGear::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.08], Wallet::new());
        ui.add_space(gap);
        ui.add_sized([width, height * 0.48], Inventory::new());
    });
}

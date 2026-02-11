use crate::egui::{self, Rect, TextureId};
use crate::styles::UiStyle;

use super::{EquipmentCard, InventoryCell, ItemCard, WeaponCard};

/// Tooltip data for an inventory item.
pub enum InventoryTooltip {
    Weapon {
        name: String,
        kind: String,
        attack: String,
        damage: String,
        range: String,
        condition: String,
    },
    Equipment {
        name: String,
        slot: String,
        description: String,
        armor: i32,
        effects: Vec<String>,
    },
    Item {
        name: String,
        description: String,
    },
}

impl InventoryTooltip {
    pub fn name(&self) -> &str {
        match self {
            Self::Weapon { name, .. }
            | Self::Equipment { name, .. }
            | Self::Item { name, .. } => name,
        }
    }
}

/// Action triggered from a cell context menu.
#[derive(Clone, Copy)]
pub enum CellAction {
    /// The primary action button was clicked (e.g. "Equip" / "Unequip").
    Primary(usize),
    /// The "Remove" button was clicked.
    Remove(usize),
}

/// A grid of [`InventoryCell`] items with configurable column and row counts.
/// Supports optional tooltip data per cell with hover popups.
pub struct InventoryTable {
    image: TextureId,
    cols: usize,
    rows: usize,
    items: Vec<Option<InventoryTooltip>>,
    id_salt: egui::Id,
    context_label: Option<String>,
    show_remove: bool,
}

impl InventoryTable {
    pub fn new(image: TextureId, cols: usize, rows: usize) -> Self {
        Self {
            image,
            cols,
            rows,
            items: Vec::new(),
            id_salt: egui::Id::NULL,
            context_label: None,
            show_remove: false,
        }
    }

    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_salt = egui::Id::new(salt);
        self
    }

    pub fn items(mut self, items: Vec<Option<InventoryTooltip>>) -> Self {
        self.items = items;
        self
    }

    pub fn context_label(mut self, label: impl Into<String>) -> Self {
        self.context_label = Some(label.into());
        self
    }

    pub fn show_remove(mut self, show: bool) -> Self {
        self.show_remove = show;
        self
    }

    /// Paints the grid into the given rect.
    /// Returns a [`CellAction`] if a context menu action was triggered.
    pub fn paint(&self, ui: &mut egui::Ui, rect: Rect) -> Option<CellAction> {
        let pad = UiStyle::content_padding(ui);
        let cell_width = (rect.width() - pad * (self.cols as f32 + 1.0)) / self.cols as f32;
        let cell_height = (rect.height() - pad * (self.rows as f32 + 1.0)) / self.rows as f32;

        let has_context = self.context_label.is_some() || self.show_remove;
        let mut action = None;

        for i in 0..(self.cols * self.rows) {
            let col = i % self.cols;
            let row = i / self.cols;
            let x = rect.min.x + pad + (cell_width + pad) * col as f32;
            let y = rect.min.y + pad + (cell_height + pad) * row as f32;
            let cell_rect =
                Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_width, cell_height));

            let has_item = self.items.get(i).is_some_and(|o| o.is_some());

            let sense = if has_context && has_item {
                egui::Sense::click() | egui::Sense::hover()
            } else {
                egui::Sense::hover()
            };

            let response = ui.interact(cell_rect, self.id_salt.with(("inv_cell", i)), sense);

            InventoryCell::new(self.image).paint(ui.painter(), cell_rect);

            if has_item {
                let dot_radius = 2.5;
                let dot_center = egui::pos2(
                    cell_rect.center().x,
                    cell_rect.max.y - cell_rect.height() * 0.12,
                );
                ui.painter()
                    .circle_filled(dot_center, dot_radius, crate::colors::STROKE_COLOR);
            }

            let mut menu_open = false;
            if has_context && has_item {
                menu_open = response
                    .context_menu(|ui| {
                        if let Some(label) = &self.context_label {
                            if ui.button(label.as_str()).clicked() {
                                action = Some(CellAction::Primary(i));
                                ui.close();
                            }
                        }
                        if self.show_remove {
                            if ui.button("Remove").clicked() {
                                action = Some(CellAction::Remove(i));
                                ui.close();
                            }
                        }
                    })
                    .is_some();
            }

            if response.hovered() && !menu_open {
                if let Some(Some(tooltip)) = self.items.get(i) {
                    let pos = response.hover_pos().unwrap_or(cell_rect.right_bottom())
                        + egui::vec2(8.0, 8.0);
                    let id = response.id;
                    match tooltip {
                        InventoryTooltip::Weapon {
                            name,
                            kind,
                            attack,
                            damage,
                            range,
                            condition,
                        } => {
                            WeaponCard::new(name)
                                .kind(kind)
                                .attack(attack)
                                .damage(damage)
                                .range(range)
                                .condition(condition)
                                .show_at(ui.ctx(), id, pos);
                        }
                        InventoryTooltip::Equipment {
                            name,
                            slot,
                            description,
                            armor,
                            effects,
                        } => {
                            EquipmentCard::new(name)
                                .slot(slot)
                                .description(description)
                                .armor(*armor)
                                .effects(effects.clone())
                                .show_at(ui.ctx(), id, pos);
                        }
                        InventoryTooltip::Item { name, description } => {
                            ItemCard::new(name)
                                .description(description)
                                .show_at(ui.ctx(), id, pos);
                        }
                    }
                }
            }
        }

        action
    }
}

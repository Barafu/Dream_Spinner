use crate::dreams::*;

pub const DREAM_ID: DreamId = "solid_color";
pub const DREAM_NAME: &'static str = "Solid Color";

/// This dream is intended to be as primitive as possible to serve as example
/// of how to implement Dream trait.
///
pub struct SolidColorDream {
    dream_settings: SolidColorSettings,
}

#[derive(PartialEq, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SolidColorSettings {
    color: egui::Color32, // The color of the background
}

impl Default for SolidColorSettings {
    fn default() -> Self {
        Self { color: egui::Color32::from_hex("#635147").unwrap() }
    }
}

impl Dream for SolidColorDream {
    fn new() -> Self {
        let mut d = Self { dream_settings: SolidColorSettings::default() };
        let txt = SETTINGS
            .read()
            .unwrap()
            .dream_settings
            .get(DREAM_ID)
            .cloned()
            .unwrap_or_default();
        d.dream_settings = toml::from_str(&txt).unwrap_or_default();
        d
    }
    fn id(&self) -> DreamId {
        DREAM_ID
    }

    fn name(&self) -> &'static str {
        DREAM_NAME
    }

    fn preferred_update_rate(&self) -> DreamUpdateRate {
        DreamUpdateRate::Fixed(5.0)
    }

    fn dream_egui(&self, ui: &mut egui::Ui) {
        // This is how to create a painter that covers the whole screen.
        // Most dreams should use that.
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        // Just fill the whole painter with color.
        painter.rect_filled(
            ui.available_rect_before_wrap(),
            0.0,
            self.dream_settings.color,
        );
    }

    fn config_egui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Color: ");
            ui.color_edit_button_srgba(&mut self.dream_settings.color);
        });
    }

    fn store(&self) {
        let txt = toml::to_string(&self.dream_settings).unwrap();
        SETTINGS
            .write()
            .unwrap()
            .dream_settings
            .insert(DREAM_ID.to_string(), txt);
    }
}

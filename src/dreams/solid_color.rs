use crate::dreams::*;

/// This dream is intended to be as primitive as possible to serve as example
/// of how to implement Dream trait.
pub struct SolidColorDream {
    stored_settings: SolidColorSettings,
    app_settings: Settings,
    color: egui::Color32,
}

#[derive(PartialEq, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SolidColorSettings {
    color_hex: String, // Stored as hex, because Color32 is not serializable
}

impl Default for SolidColorSettings {
    fn default() -> Self {
        Self { color_hex: "#635147".to_string() }
    }
}

impl Dream for SolidColorDream {
    fn new(settings: Settings) -> Self {
        Self {
            stored_settings: SolidColorSettings::default(),
            app_settings: settings,
            color: egui::Color32::BLACK,
        }
    }
    fn id(&self) -> DreamId {
        "solid_color".to_string()
    }

    fn name(&self) -> String {
        "Solid Color".to_string()
    }

    fn get_type(&self) -> DreamType {
        return DreamType::Egui;
    }

    fn dream_egui(&self, ui: &mut egui::Ui) {
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        painter.rect_filled(ui.available_rect_before_wrap(), 0.0, self.color);
    }

    fn config_egui(&mut self, ui: &mut egui::Ui) {
        ui.color_edit_button_srgba(&mut self.color);
    }

    fn prepare(&mut self) {
        let txt = self
            .app_settings
            .read()
            .unwrap()
            .dream_settings
            .get(&self.id())
            .cloned()
            .unwrap_or_default();
        self.stored_settings = toml::from_str(&txt).unwrap_or_default();
        self.color = egui::Color32::from_hex(&self.stored_settings.color_hex)
            .unwrap_or_default();
    }

    fn store(&self) {
        let mut s = self.stored_settings.clone();
        s.color_hex = self.color.to_hex();
        let txt = toml::to_string(&s).unwrap();
        self.app_settings
            .write()
            .unwrap()
            .dream_settings
            .insert(self.id(), txt);
    }
}

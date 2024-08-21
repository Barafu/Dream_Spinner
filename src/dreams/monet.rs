use crate::dreams::*;

pub struct MonetDream {
    dream_settings: MonetSettings,
}

#[derive(PartialEq, Debug, Default, serde::Deserialize, serde::Serialize)]
struct MonetSettings {}

impl Dream for MonetDream {
    fn new() -> Self
    where
        Self: Sized,
    {
        let local_settings = MonetSettings::default();
        let mut d = Self { dream_settings: local_settings };
        let txt = SETTINGS
            .read()
            .unwrap()
            .dream_settings
            .get(&d.id())
            .cloned()
            .unwrap_or_default();
        d.dream_settings = toml::from_str(&txt).unwrap_or_default();
        d
    }

    fn id(&self) -> super::DreamId {
        "monet".to_string()
    }

    fn name(&self) -> String {
        "Monet".to_string()
    }

    fn get_type(&self) -> super::DreamType {
        return super::DreamType::Egui;
    }

    fn dream_egui(&self, _ui: &mut egui::Ui) {}

    fn config_egui(&mut self, _ui: &mut egui::Ui) {}
}

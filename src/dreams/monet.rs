use crate::dreams::*;

pub const DREAM_ID: DreamId = "monet";
pub const DREAM_NAME: &'static str = "Monet";

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
            .get(DREAM_ID)
            .cloned()
            .unwrap_or_default();
        d.dream_settings = toml::from_str(&txt).unwrap_or_default();
        d
    }

    fn id(&self) -> super::DreamId {
        DREAM_ID
    }

    fn name(&self) -> &'static str {
        DREAM_NAME
    }

    fn in_development(&self) -> bool {
        true
    }

    fn preferred_update_rate(&self) -> super::DreamUpdateRate {
        super::DreamUpdateRate::Fixed(0.5)
    }

    fn dream_egui(&self, _ui: &mut egui::Ui) {
        unimplemented!()
    }

    fn config_egui(&mut self, _ui: &mut egui::Ui) {}
}

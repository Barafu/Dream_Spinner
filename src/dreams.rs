use crate::app_settings::Settings;
use std::sync::{Arc, RwLock};

mod solid_color;
mod dendraclock;

pub enum DreamType {
    Egui,
}
pub trait Dream: Sync + Send {
    /// Create the dream using the settings
    fn new(settings: Arc<RwLock<Settings>>) -> Self
    where
        Self: Sized;
    /// Gives unique ID for the system
    fn id(&self) -> String;
    /// Gives the name to display in UI
    fn name(&self) -> String;

    /// Prepare dream for rendering (load resources, initialize RNG etc.)    
    fn prepare(&self) {}

    /// Return true if prepare() takes noticeable time enough to warrant a loading screen
    fn needs_loading(&self) -> bool {
        false
    }

    /// Dream type determines what kind of window to perpare for it.
    fn get_type(&self) -> DreamType;

    /// Draws the dream in egui. This function MUST be thread-safe.
    fn dream_egui(&mut self, _ui: &mut egui::Ui) {
        unimplemented!("EGUI rendering called, but not implemented");
    }

    /// Show the config window in egui;
    fn config_egui(&mut self, _ui: &mut egui::Ui) {
        unimplemented!("EGUI config called, but not implemented");
    }

    /// Makes dream to serialise its config and strore it in Settings.
    fn store(&self)  { }
}

/// For giggles, I call the collection of all dream types "zoo"
pub fn build_zoo(settings: Arc<RwLock<Settings>>) -> Vec<Arc<RwLock<dyn Dream>>> {
    let mut zoo: Vec<Arc<RwLock<dyn Dream>>> = Vec::new();
    let d = RwLock::new(solid_color::SolidColorDream::new(settings.clone()));
    zoo.push(Arc::new(d));
    let d = RwLock::new(dendraclock::DendraClock::new(settings.clone()));
    zoo.push(Arc::new(d));
    zoo
}

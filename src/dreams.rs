use crate::app_settings::SETTINGS;
use std::sync::{Arc, RwLock};

mod dendraclock;
mod monet;
mod solid_color;

/// For giggles, I call the collection of all dream types "zoo"
pub type Zoo = Vec<Arc<RwLock<dyn Dream>>>;
pub type DreamId = String;

#[derive(PartialEq, Debug)]
pub enum DreamType {
    Egui,
}
pub trait Dream: Sync + Send {
    /// Create the dream using the settings
    fn new() -> Self
    where
        Self: Sized;

    /// Returns the unique ID of the dream
    ///
    /// Should be lowercase with underscores, like "dream_of_sheep"
    fn id(&self) -> DreamId;

    /// Gives the name to display in UI. The name also serves as ID, including
    /// in settings, so it must be unique
    fn name(&self) -> String;

    /// Prepare dream for rendering (load resources, initialize RNG etc.)    
    fn prepare(&mut self) {}

    /// Return true if prepare() takes noticeable time enough to warrant a loading screen
    fn needs_loading(&self) -> bool {
        false
    }

    /// Dream type determines what kind of window to perpare for it.
    fn get_type(&self) -> DreamType;

    /// Draws the dream in egui. This function MUST be thread-safe.
    fn dream_egui(&self, _ui: &mut egui::Ui) {
        unimplemented!("EGUI rendering called, but not implemented");
    }

    /// Show the config window in egui;
    fn config_egui(&mut self, _ui: &mut egui::Ui) {
        unimplemented!("EGUI config called, but not implemented");
    }

    /// Makes dream to serialise its config and strore it in Settings.
    fn store(&self) {}
}

pub fn build_zoo() -> Zoo {
    let mut zoo: Zoo = Zoo::new();
    let d = RwLock::new(solid_color::SolidColorDream::new());
    zoo.push(Arc::new(d));
    let d = RwLock::new(dendraclock::DendraClockDream::new());
    zoo.push(Arc::new(d));
    let d = RwLock::new(monet::MonetDream::new());
    zoo.push(Arc::new(d));
    zoo
}

// Pick a dream from zoo by its id.
pub fn select_dream_by_id(
    zoo: &Zoo,
    id: &DreamId,
) -> Option<Arc<RwLock<dyn Dream>>> {
    zoo.iter().find(|d| d.read().unwrap().id() == *id).map(|d| d.clone())
}

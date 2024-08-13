use crate::app_settings::Settings;
use std::sync::{Arc, RwLock};

mod solid_color;
mod dendraclock;

/// For giggles, I call the collection of all dream types "zoo"
pub type Zoo = Vec<Arc<RwLock<dyn Dream>>>;

#[derive(PartialEq, Debug)]
pub enum DreamType {
    Egui,
}
pub trait Dream: Sync + Send {
    /// Create the dream using the settings
    fn new(settings: Settings) -> Self
    where
        Self: Sized;
    /// Gives unique ID of the dream. Must be a unique literal between 0 and 429496729
    /// If you have Python, run
    /// ```python
    ///     python -c "import random; print (random.randrange(429496729))"
    /// ```
    fn id(&self) -> u32;
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
    fn dream_egui(&self, _ui: &mut egui::Ui) {
        unimplemented!("EGUI rendering called, but not implemented");
    }

    /// Show the config window in egui;
    fn config_egui(&mut self, _ui: &mut egui::Ui) {
        unimplemented!("EGUI config called, but not implemented");
    }

    /// Makes dream to serialise its config and strore it in Settings.
    fn store(&self)  { }
}

pub fn build_zoo(settings: Settings) -> Zoo {
    let mut zoo: Zoo = Zoo::new();
    let d = RwLock::new(solid_color::SolidColorDream::new(settings.clone()));
    zoo.push(Arc::new(d));
    let d = RwLock::new(dendraclock::DendraClockDream::new(settings.clone()));
    zoo.push(Arc::new(d));
    zoo
}

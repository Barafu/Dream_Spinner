use crate::app_settings::SETTINGS;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

mod dendraclock;
mod monet;
mod solid_color;

pub type DreamId = &'static str;
pub type ADream = Arc<RwLock<dyn Dream>>;

#[derive(PartialEq, Debug)]
pub enum DreamType {
    Egui,
}

// Shows how often a dream is intended to redraw its image. Smooth means match
// the FPS of the monitor. Fixed means delay in seconds.
pub enum DreamUpdateRate {
    Smooth,
    Fixed(f32),
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
    fn name(&self) -> &'static str;

    /// Prepare dream for rendering (load resources, initialize RNG etc.)    
    fn prepare_dream(&mut self) {}

    /// Return true if prepare() takes noticeable time enough to warrant a loading screen
    fn require_load_screen(&self) -> bool {
        false
    }

    /// Indicates that dream is in development and should not be shown to the end user.
    /// Allow dev mode in settings to see this dream.
    fn in_development(&self) -> bool {
        false
    }

    /// Returns the update rate of the dream. It shows how often a dream is intended to
    /// update, but actual updates may be required more frequently. May change
    /// dynamically.
    fn preferred_update_rate(&self) -> DreamUpdateRate;

    /// Dream type determines what kind of window to perpare for it.
    fn get_type(&self) -> DreamType {
        DreamType::Egui
    }

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

pub fn build_dreams_id_list() -> BTreeMap<&'static str, &'static str> {
    let mut zoo = BTreeMap::new();
    zoo.insert(dendraclock::DREAM_ID, dendraclock::DREAM_NAME);
    zoo.insert(monet::DREAM_ID, monet::DREAM_NAME);
    zoo.insert(solid_color::DREAM_ID, solid_color::DREAM_NAME);
    zoo
}

// Pick a dream from zoo by its id.
pub fn build_dream_by_id(id: &str) -> ADream {
    match id {
        dendraclock::DREAM_ID => {
            Arc::new(RwLock::new(dendraclock::DendraClockDream::new()))
        }
        monet::DREAM_ID => Arc::new(RwLock::new(monet::MonetDream::new())),
        solid_color::DREAM_ID => {
            Arc::new(RwLock::new(solid_color::SolidColorDream::new()))
        }
        _ => panic!("Unknown dream id: {}", id),
    }
}

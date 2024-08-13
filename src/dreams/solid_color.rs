use crate::dreams::*;



pub struct SolidColorDream {
    color: egui::Color32,
    settings: Arc<RwLock<Settings>>,
}


impl Dream for SolidColorDream {
    fn new(settings: Arc<RwLock<Settings>>) -> Self {
        Self {
            color: egui::Color32::BROWN,
            settings,
        }
    }
    fn id(&self) -> String {
        "SolidColor".to_string()
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

}

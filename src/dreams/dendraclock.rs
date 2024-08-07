use egui::{widgets::*, *};
use std::f32::consts::TAU;
use chrono::{Local, Timelike};
use crate::dreams::*;

pub struct DendraClock {
    paused: bool,
    time: f64,
    local_settings: DendraClockSettings,    
    app_settings: Arc<RwLock<Settings>>,
}

#[derive(PartialEq, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct DendraClockSettings {
    zoom: f32,
    start_line_width: f32,
    depth: usize,
    length_factor: f32,
    luminance_factor: f32,
    width_factor: f32,
    line_count: usize,
}

impl Default for DendraClockSettings {
    fn default() -> Self {
        Self {
            zoom: 0.25,
            start_line_width: 2.5,
            depth: 9,
            length_factor: 0.8,
            luminance_factor: 0.8,
            width_factor: 0.9,
            line_count: 0,
        }
    }
}

impl Dream for DendraClock {
    fn id(&self) -> String {
        "FractalClock".to_string()
    }

    fn name(&self) -> String {
        "Fractal Clock".to_string()
    }

    fn new(settings: Arc<RwLock<Settings>>) -> Self {
        let local_settings = DendraClockSettings::default();
        Self {
            paused: false,
            time: 0.0,
            local_settings,
            app_settings: settings,
        }
    }

    fn get_type(&self) -> DreamType {
        return DreamType::Egui;
    }

    fn dream_egui(&mut self, ui: &mut egui::Ui) {
        self.paint_ui(ui);
    }

    fn config_egui(&mut self, ui: &mut egui::Ui) {
        self.options_ui(ui);
    }
    
    fn prepare(&self) {}
    
    fn needs_loading(&self) -> bool {
        false
    }
    
    fn store(&self)  { }
}

impl DendraClock {
    pub fn paint_ui(&mut self, ui: &mut Ui) {
        let now = Local::now().time();
        self.time =
        now.num_seconds_from_midnight() as f64 + now.nanosecond() as f64 * 1e-9;
        
        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        self.paint(&painter);
        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        // Tell UI to paint next frame as soon as possible.
        /*if !self.paused {
            ui.ctx().request_repaint();
        }*/
    }

    fn options_ui(&mut self, ui: &mut Ui) {
        //ui.label(format!("Local time: {:02}:{:02}:{:02}.{:03}", (self.time % (24.0 * 60.0 * 60.0) / 3600.0).floor(), (self.time % (60.0 * 60.0) / 60.0).floor(), (self.time % 60.0).floor(), (self.time % 1.0 * 100.0).floor()));
        /*if seconds_since_midnight.is_some() {
            ui.label(format!(
                "Local time: {:02}:{:02}:{:02}.{:03}",
                (self.time % (24.0 * 60.0 * 60.0) / 3600.0).floor(),
                (self.time % (60.0 * 60.0) / 60.0).floor(),
                (self.time % 60.0).floor(),
                (self.time % 1.0 * 100.0).floor()
            ));
        } else {
            ui.label("The fractal_clock clock is not showing the correct time");
        };*/
       
        ui.add(Slider::new(&mut self.local_settings.zoom, 0.0..=1.0).text("zoom"));
        ui.add(Slider::new(&mut self.local_settings.start_line_width, 0.0..=5.0).text("Start line width"));
        ui.add(Slider::new(&mut self.local_settings.depth, 0..=14).text("depth"));
        ui.add(Slider::new(&mut self.local_settings.length_factor, 0.0..=1.0).text("length factor"));
        ui.add(Slider::new(&mut self.local_settings.luminance_factor, 0.0..=1.0).text("luminance factor"));
        ui.add(Slider::new(&mut self.local_settings.width_factor, 0.0..=1.0).text("width factor"));
        
        egui::reset_button(ui, &mut self.local_settings, "Reset");

        ui.hyperlink_to(
            "Inspired by a screensaver by Rob Mayoff",
            "http://www.dqd.com/~mayoff/programs/FractalClock/",
        );
    }

    fn paint(&mut self, painter: &Painter) {
        struct Hand {
            length: f32,
            angle: f32,
            vec: Vec2,
        }

        impl Hand {
            fn from_length_angle(length: f32, angle: f32) -> Self {
                Self {
                    length,
                    angle,
                    vec: length * Vec2::angled(angle),
                }
            }
        }

        let angle_from_period =
            |period| TAU * (self.time.rem_euclid(period) / period) as f32 - TAU / 4.0;

        let hands = [
            // Second hand:
            Hand::from_length_angle(self.local_settings.length_factor, angle_from_period(60.0)),
            // Minute hand:
            Hand::from_length_angle(self.local_settings.length_factor, angle_from_period(60.0 * 60.0)),
            // Hour hand:
            Hand::from_length_angle(0.5, angle_from_period(12.0 * 60.0 * 60.0)),
        ];

        let mut shapes: Vec<Shape> = Vec::new();

        let rect = painter.clip_rect();
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(Pos2::ZERO, rect.square_proportions() / self.local_settings.zoom),
            rect,
        );

        let mut paint_line = |points: [Pos2; 2], color: Color32, width: f32| {
            let line = [to_screen * points[0], to_screen * points[1]];

            // culling
            if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
                shapes.push(Shape::line_segment(line, (width, color)));
            }
        };

        let hand_rotations = [
            hands[0].angle - hands[2].angle + TAU / 2.0,
            hands[1].angle - hands[2].angle + TAU / 2.0,
        ];

        let hand_rotors = [
            hands[0].length * emath::Rot2::from_angle(hand_rotations[0]),
            hands[1].length * emath::Rot2::from_angle(hand_rotations[1]),
        ];

        #[derive(Clone, Copy)]
        struct Node {
            pos: Pos2,
            dir: Vec2,
        }

        let mut nodes = Vec::with_capacity(self.local_settings.line_count);

        let mut width = self.local_settings.start_line_width;

        for (i, hand) in hands.iter().enumerate() {
            let center = pos2(0.0, 0.0);
            let end = center + hand.vec;
            paint_line([center, end], Color32::from_additive_luminance(255), width);
            if i < 2 {
                nodes.push(Node {
                    pos: end,
                    dir: hand.vec,
                });
            }
        }

        let mut luminance = 0.7; // Start dimmer than main hands

        let mut new_nodes = Vec::new();
        for _ in 0..self.local_settings.depth {
            new_nodes.clear();
            new_nodes.reserve(nodes.len() * 2);

            luminance *= self.local_settings.luminance_factor;
            width *= self.local_settings.width_factor;

            let luminance_u8 = (255.0 * luminance).round() as u8;
            if luminance_u8 == 0 {
                break;
            }

            for &rotor in &hand_rotors {
                for a in &nodes {
                    let new_dir = rotor * a.dir;
                    let b = Node {
                        pos: a.pos + new_dir,
                        dir: new_dir,
                    };
                    paint_line(
                        [a.pos, b.pos],
                        Color32::from_additive_luminance(luminance_u8),
                        width,
                    );
                    new_nodes.push(b);
                }
            }

            std::mem::swap(&mut nodes, &mut new_nodes);
        }
        self.local_settings.line_count = shapes.len();
        painter.extend(shapes);
    }
}
use crate::{app_settings::ColorScheme, dreams::*};
use chrono::{Local, Timelike};
use egui::{widgets::*, *};
use std::f32::consts::TAU;

pub const DREAM_ID: DreamId = "fractal_clock";
pub const DREAM_NAME: &'static str = "Fractal Clock";

pub struct DendraClockDream {
    dream_settings: DendraClockSettings,
    color_scheme: ColorScheme,
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

impl Dream for DendraClockDream {
    fn new() -> Self {
        let local_settings = DendraClockSettings::default();
        let color_scheme = SETTINGS.read().unwrap().color_scheme.clone();
        let mut d = Self { dream_settings: local_settings, color_scheme };
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

    fn get_type(&self) -> DreamType {
        return DreamType::Egui;
    }

    fn preferred_update_rate(&self) -> DreamUpdateRate {
        DreamUpdateRate::Smooth
    }

    fn dream_egui(&self, ui: &mut egui::Ui) {
        self.paint_ui(ui);
    }

    fn config_egui(&mut self, ui: &mut egui::Ui) {
        self.options_ui(ui);
    }

    fn prepare_dream(&mut self) {}

    fn store(&self) {
        let txt = toml::to_string(&self.dream_settings).unwrap();
        SETTINGS
            .write()
            .unwrap()
            .dream_settings
            .insert(DREAM_ID.to_string(), txt);
    }
}

impl DendraClockDream {
    pub fn paint_ui(&self, ui: &mut Ui) {
        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        self.paint(&painter);
        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());
    }

    fn options_ui(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.dream_settings.zoom, 0.0..=1.0).text("zoom"),
        );
        ui.add(
            Slider::new(&mut self.dream_settings.start_line_width, 0.0..=5.0)
                .text("Start line width"),
        );
        ui.add(
            Slider::new(&mut self.dream_settings.depth, 0..=14).text("depth"),
        );
        ui.add(
            Slider::new(&mut self.dream_settings.length_factor, 0.0..=1.0)
                .text("length factor"),
        );
        ui.add(
            Slider::new(&mut self.dream_settings.luminance_factor, 0.0..=1.0)
                .text("luminance factor"),
        );
        ui.add(
            Slider::new(&mut self.dream_settings.width_factor, 0.0..=1.0)
                .text("width factor"),
        );

        egui::reset_button(ui, &mut self.dream_settings, "Reset");

        ui.hyperlink_to(
            "Inspired by a screensaver by Rob Mayoff",
            "http://www.dqd.com/~mayoff/programs/FractalClock/",
        );
    }

    fn paint(&self, painter: &Painter) {
        struct Hand {
            length: f32,
            angle: f32,
            vec: Vec2,
        }

        impl Hand {
            fn from_length_angle(length: f32, angle: f32) -> Self {
                Self { length, angle, vec: length * Vec2::angled(angle) }
            }
        }

        let now = Local::now().time();
        let time = now.num_seconds_from_midnight() as f64
            + now.nanosecond() as f64 * 1e-9;
        let angle_from_period = |period| {
            TAU * (time.rem_euclid(period) / period) as f32 - TAU / 4.0
        };

        let hands = [
            // Second hand:
            Hand::from_length_angle(
                self.dream_settings.length_factor,
                angle_from_period(60.0),
            ),
            // Minute hand:
            Hand::from_length_angle(
                self.dream_settings.length_factor,
                angle_from_period(60.0 * 60.0),
            ),
            // Hour hand:
            Hand::from_length_angle(0.5, angle_from_period(12.0 * 60.0 * 60.0)),
        ];

        let mut shapes: Vec<Shape> = Vec::new();

        let rect = painter.clip_rect();
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(
                Pos2::ZERO,
                rect.square_proportions() / self.dream_settings.zoom,
            ),
            rect,
        );

        let background = Shape::rect_filled(
            rect,
            Rounding::ZERO,
            self.color_scheme.background,
        );
        shapes.push(background);

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

        let mut nodes = Vec::with_capacity(self.dream_settings.line_count);

        let mut width = self.dream_settings.start_line_width;

        for (i, hand) in hands.iter().enumerate() {
            let center = pos2(0.0, 0.0);
            let end = center + hand.vec;
            paint_line([center, end], self.color_scheme.foreground, width);
            if i < 2 {
                nodes.push(Node { pos: end, dir: hand.vec });
            }
        }

        let mut luminance = 0.7; // Start dimmer than main hands

        let mut new_nodes = Vec::new();
        for depth in 0..self.dream_settings.depth {
            new_nodes.clear();
            new_nodes.reserve(nodes.len() * 2);

            luminance *= self.dream_settings.luminance_factor;
            width *= self.dream_settings.width_factor;

            let depth_color =
                self.color_scheme.foreground.gamma_multiply(luminance);

            for &rotor in &hand_rotors {
                for a in &nodes {
                    let new_dir = rotor * a.dir;
                    let b = Node { pos: a.pos + new_dir, dir: new_dir };
                    paint_line([a.pos, b.pos], depth_color, width);
                    new_nodes.push(b);
                }
            }

            std::mem::swap(&mut nodes, &mut new_nodes);
        }
        painter.extend(shapes);
    }
}

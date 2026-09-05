use midir::{MidiOutput, MidiOutputPort};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MidiSettings {
    #[serde(skip)]
    pub midi_out_ports: Vec<String>,
    pub selected_midi: String,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Input2MicrocosmApp {
    midi_settings: MidiSettings,
    #[serde(skip)]
    midi_out: MidiOutput,
}

impl Default for Input2MicrocosmApp {
    fn default() -> Self {
        let midi_out = MidiOutput::new("input2microcosm").unwrap();
        
        Self {
            midi_out: midi_out,
            midi_settings: MidiSettings {
                midi_out_ports: Vec::new(),
                selected_midi: String::new(),
            },
        }
    }
}

impl Input2MicrocosmApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: Input2MicrocosmApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app.refresh_midi_ports();
        app
    }

    fn refresh_midi_ports(&mut self) {
        self.midi_settings.midi_out_ports = self
            .midi_out
            .ports()
            .iter()
            .map(|p| self.midi_out.port_name(p).unwrap())
            .collect();
    }
}

impl eframe::App for Input2MicrocosmApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show(ui, |ui| {            
            ui.horizontal(|ui| {
                ui.label("Select MIDI Output: ");
                egui::ComboBox::from_id_salt("MIDI picker")
                    .selected_text(&self.midi_settings.selected_midi)
                    .show_ui(ui, |ui| {
                        self.midi_settings.midi_out_ports.iter().for_each(|p| {
                            ui.selectable_value(
                                &mut self.midi_settings.selected_midi,
                                p.clone(),
                                p,
                            );
                        });
                    });
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if ui.button("Start").clicked() {
                println!("wow");
            }
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

use std::collections::HashMap;

use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::microcosm_signal::MicrocosmSignal;

#[derive(serde::Deserialize, serde::Serialize)]
struct MidiSettings {
    #[serde(skip)]
    pub midi_out_ports: HashMap<String, MidiOutputPort>,
    pub selected_midi: String,
    #[serde(skip)]
    pub connected: bool,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Input2MicrocosmApp {
    midi_settings: MidiSettings,
    #[serde(skip)]
    midi_out: Option<MidiOutput>,
    #[serde(skip)]
    send_midi: bool,
    #[serde(skip)]
    active_states: HashMap<MicrocosmSignal, bool>,
    #[serde(skip)]
    midi_out_connection: Option<MidiOutputConnection>,
}

impl Default for Input2MicrocosmApp {
    fn default() -> Self {
        Self {
            midi_settings: MidiSettings {
                midi_out_ports: HashMap::new(),
                selected_midi: String::new(),
                connected: false,
            },
            midi_out: Some(MidiOutput::new("input2microcosm").unwrap()),
            send_midi: false,
            active_states: HashMap::new(),
            midi_out_connection: None,
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
        app.connect();
        app
    }

    fn refresh_midi_ports(&mut self) {
        let midi_out = self.midi_out.as_ref().unwrap();

        self.midi_settings.midi_out_ports = midi_out
            .ports()
            .iter()
            .map(|p| (midi_out.port_name(p).unwrap(), p.clone()))
            .collect()
    }

    fn send_signals_out(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::A) {
                self.send_signal(&MicrocosmSignal::LooperRecord);
            }
            if i.key_pressed(egui::Key::B) {
                self.send_signal(&MicrocosmSignal::LooperPlay);
            }
            if i.key_pressed(egui::Key::C) {
                self.send_signal(&MicrocosmSignal::LooperOverdub);
            }
        })
    }

    fn is_active(&self, signal: &MicrocosmSignal) -> &bool {
        self.active_states.get(signal).unwrap_or_else(|| &false)
    }

    fn send_signal(&mut self, signal: &MicrocosmSignal) {
        let conn = self.midi_out_connection.as_mut().unwrap();
        
        let signal = signal.clone();
        let _ = conn.send(&[0xB0, signal as u8, 0x7F]);
    }

    fn connect(&mut self) -> Result<(), String> {
        if self.midi_settings.selected_midi != "" {
            let midi_out = self.midi_out.take().ok_or_else(|| "MIDI not available")?;
            let port = self
                .midi_settings
                .midi_out_ports
                .get(&self.midi_settings.selected_midi)
                .unwrap();
            let connection = midi_out.connect(port, "input2microcosm").unwrap();
            self.midi_out_connection = Some(connection);
            self.midi_settings.connected = true;
        }

        Ok(())
    }
}

impl eframe::App for Input2MicrocosmApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.midi_settings.connected {
            self.connect();
        }

        if self.send_midi {
            self.send_signals_out(ctx);
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Select MIDI Output: ");
                let current_val = self.midi_settings.selected_midi.clone();
                egui::ComboBox::from_id_salt("MIDI picker")
                    .selected_text(&self.midi_settings.selected_midi)
                    .show_ui(ui, |ui| {
                        self.midi_settings.midi_out_ports.iter().for_each(|p| {
                            ui.selectable_value(
                                &mut self.midi_settings.selected_midi,
                                p.0.clone(),
                                p.0,
                            );
                        });
                    });

                if &self.midi_settings.selected_midi != &current_val {
                    self.connect();
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(110.0)
                    .show(ui, |ui| {
                        let button_text = if self.send_midi { "Stop" } else { "Start" };
                        if ui
                            .add_enabled(
                                self.midi_settings.connected,
                                egui::Button::new(button_text),
                            )
                            .clicked()
                        {
                            self.send_midi = !self.send_midi;
                        };
                    });
            });
        });
    }
}

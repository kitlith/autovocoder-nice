use egui::{Margin, Vec2};
use nice_plug::editor::Editor;
use nice_plug_egui::{create_egui_editor, resizable_window::ResizableWindow, widgets::generic_ui};

use crate::{NiceAutoVocoder, presets::PRESETS};

pub const MIN_WINDOW_WIDTH: u32 = 300;
pub const MIN_WINDOW_HEIGHT: u32 = 300;

struct GuiState {
    preset_idx: usize,
}

impl NiceAutoVocoder {
    pub(crate) fn editor_impl(&mut self) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            GuiState { preset_idx: 0 },
            Default::default(),
            |_egui_ctx, _queue, _gui_state| {},
            move |ui, setter, _queue, gui_state| {
                ResizableWindow::new("res-wind")
                    .min_size(Vec2::new(MIN_WINDOW_WIDTH as f32, MIN_WINDOW_HEIGHT as f32))
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(Margin::same(5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    egui::ComboBox::from_label("Preset").show_index(
                                        ui,
                                        &mut gui_state.preset_idx,
                                        PRESETS.len(),
                                        |idx| PRESETS[idx].0,
                                    );

                                    if ui.button("Apply").clicked() {
                                        PRESETS[gui_state.preset_idx].1.apply(&params, setter);
                                    }
                                });

                                generic_ui::create(
                                    ui,
                                    params.clone(),
                                    setter,
                                    generic_ui::GenericSlider,
                                );
                            });
                    });
            },
        )
    }
}

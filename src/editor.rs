use std::sync::Arc;

use egui::{Layout, Margin, TextStyle, Ui, Vec2};
use nice_plug::{
    context::gui::ParamSetter,
    editor::Editor,
    params::{BoolParam, Param, ParamFlags, Params, enums::EnumParamInner, internals::ParamPtr},
};
use nice_plug_egui::{create_egui_editor, resizable_window::ResizableWindow, widgets::ParamSlider};

use crate::{NiceAutoVocoder, presets::PRESETS};

pub const MIN_WINDOW_WIDTH: u32 = 500;
pub const MIN_WINDOW_HEIGHT: u32 = 300;

struct GenericUi;

// based on nice_plug_egui::generic_ui
impl GenericUi {
    /// The same as [`add_widget()`][Self::add_widget()], but for a `ParamPtr`.
    ///
    /// # Safety
    ///
    /// Undefined behavior of the `ParamPtr` does not point to a valid parameter.
    unsafe fn add_widget_raw(ui: &mut Ui, param: &ParamPtr, setter: &ParamSetter) {
        unsafe {
            match param {
                ParamPtr::FloatParam(p) => Self::add_slider(ui, &**p, setter),
                ParamPtr::IntParam(p) => Self::add_slider(ui, &**p, setter),
                ParamPtr::BoolParam(p) => Self::add_checkbox(ui, &**p, setter),
                ParamPtr::EnumParam(p) => Self::add_dropdown(ui, &**p, setter),
            }
        }
    }

    fn add_slider<P: Param>(ui: &mut Ui, param: &P, setter: &ParamSetter) {
        // ui.label(param.name());
        // Make these sliders a bit wider, else they look a bit odd
        ui.add(ParamSlider::for_param(param, setter).with_width(100.0));
    }

    fn add_dropdown(ui: &mut Ui, param: &EnumParamInner, setter: &ParamSetter) {
        // ui.label(param.name());

        let mut current = param.modulated_plain_value() as usize;
        let response = egui::ComboBox::from_id_salt(param.name()).show_index(
            ui,
            &mut current,
            param.len(),
            |idx| param.normalized_value_to_string(param.preview_normalized(idx as i32), false),
        );
        // TODO: call begin_set_parameter/end_set_parameter when dropdown opens/closes?
        if response.changed() {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, current as i32);
            setter.end_set_parameter(param);
        }
    }

    fn add_checkbox(ui: &mut Ui, param: &BoolParam, setter: &ParamSetter) {
        // ui.label(param.name());

        let mut current = param.value();
        if ui.checkbox(&mut current, ()).changed() {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, current);
            setter.end_set_parameter(param);
        }
    }

    /// Create a scrollable generic UI using the specified widget. Takes up all the remaining vertical
    /// space.
    pub fn create(ui: &mut Ui, params: Arc<impl Params>, setter: &ParamSetter) {
        egui::containers::ScrollArea::vertical()
            // Take up all remaining space, use a wrapper container to adjust how much space that is
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("generic_ui_grid").show(ui, |ui| {
                    for (_, param_ptr, _) in params.param_map().into_iter() {
                        let flags = unsafe { param_ptr.flags() };
                        if flags.contains(ParamFlags::HIDE_IN_GENERIC_UI) {
                            continue;
                        }

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| unsafe {
                            ui.label(param_ptr.name())
                        });

                        unsafe { Self::add_widget_raw(ui, &param_ptr, setter) };

                        ui.end_row();
                    }
                })
            });
    }
}

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
                                    ui.with_layout(
                                        Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            egui::ComboBox::from_id_salt("Preset").show_index(
                                                ui,
                                                &mut gui_state.preset_idx,
                                                PRESETS.len(),
                                                |idx| PRESETS[idx].0,
                                            );

                                            if ui.button("Apply Preset").clicked() {
                                                PRESETS[gui_state.preset_idx]
                                                    .1
                                                    .apply(&params, setter);
                                            }
                                        },
                                    )
                                });

                                let padding =
                                    Vec2::splat(ui.text_style_height(&TextStyle::Body) * 0.2);
                                ui.allocate_space(padding);

                                GenericUi::create(ui, params.clone(), setter);
                            });
                    });
            },
        )
    }
}

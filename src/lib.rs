use nice_plug::prelude::*;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, RepaintNotifier, create_egui_editor,
};
use std::sync::{Arc, atomic::Ordering::SeqCst};
use tracing::level_filters::LevelFilter;

use autovocoder_dsp::{AutoVocoder, AutoVocoderConfig};

mod editor;
mod enums;
mod params;
mod presets;

use params::*;

use crate::{
    editor::{MIN_WINDOW_SIZE, NiceAutoVocoderEditor, RESIZE_HINT},
    params::UpdateFlags,
};

pub struct NiceAutoVocoder {
    params: Arc<AutoVocoderParams>,
    editor_state: Arc<EguiEditorState>,
    repaint_notifier: RepaintNotifier,
    inner: Option<AutoVocoder>,
    update_flags: Arc<atomig::Atomic<UpdateFlags>>,
    input_scratch: Vec<f32>,
}

// impl From<AutoVocoderParams> for AutoVocoderConfig

impl Default for NiceAutoVocoder {
    fn default() -> Self {
        let update_flags = Arc::new(atomig::Atomic::default());
        Self {
            params: Arc::new(AutoVocoderParams::new(update_flags.clone())),
            editor_state: EguiEditorState::from_size(MIN_WINDOW_SIZE, 1.0),
            repaint_notifier: RepaintNotifier::new(),
            inner: None,
            update_flags,
            input_scratch: Vec::new(),
        }
    }
}

impl Plugin for NiceAutoVocoder {
    const NAME: &'static str = "Autovocoder (nice-plug)";
    const VENDOR: &'static str = "hotspoons + kitlith";
    const URL: &'static str = "https://github.com/kitlith/autovocoder-nice";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // AudioIOLayout {
        //     main_input_channels: NonZeroU32::new(2),
        //     main_output_channels: NonZeroU32::new(2),
        //     aux_input_ports: &[],
        //     aux_output_ports: &[],
        //     names: PortNames::const_default(),
        // },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();
    type Editor = EguiEditor<NiceAutoVocoderEditor>;

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        // autovocoder needs to be recreated when sample rate changes.
        let needs_creation = self
            .inner
            .as_ref()
            .map(|av| av.sample_rate() != buffer_config.sample_rate)
            .unwrap_or(true);
        if needs_creation {
            let mut av = AutoVocoder::new(buffer_config.sample_rate, AutoVocoderConfig::default());

            // Clear pending updates
            // self.update_flags.store(UpdateFlags::default(), SeqCst);

            //info!("{:?}", &self.params);

            // Reinitialize parameters.
            UpdateFlags::all().update(&mut av, &self.params);

            self.inner = Some(av);
        }

        if self.input_scratch.len() < buffer_config.max_buffer_size as usize {
            self.input_scratch
                .resize(buffer_config.max_buffer_size as usize, 0.);
        }
        true
    }

    fn reset(&mut self) {
        if let Some(av) = &mut self.inner {
            av.reset();
        }
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        macro_rules! err {
            ($msg:literal) => {{
                return ProcessStatus::Error($msg);
            }};
        }

        let inner = match &mut self.inner {
            Some(av) => av,
            None => err!("inner plugin not initialized, was initialize not called?"),
        };

        let updated_params = self.update_flags.swap(UpdateFlags::default(), SeqCst);
        if updated_params != UpdateFlags::default() {
            updated_params.update(inner, &self.params);
        }

        if buffer.channels() != 1 {
            err!("Currently only supports a single channel!");
        }

        for (_, mut block) in buffer.iter_blocks(self.input_scratch.len()) {
            let output_buf = block.get_mut(0).unwrap();
            let input_buf = &mut self.input_scratch[..output_buf.len()];
            input_buf.copy_from_slice(output_buf);
            inner.process_block(input_buf, output_buf);
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {
        self.inner = None;
        self.input_scratch = Vec::new();
    }

    fn setup_logger() -> Option<bool> {
        Some(
            tracing::subscriber::set_global_default(
                tracing_subscriber::FmtSubscriber::builder()
                    .with_max_level(if cfg!(debug_assertions) {
                        LevelFilter::DEBUG
                    } else {
                        LevelFilter::INFO
                    })
                    // This custom writer reads from the `NICE_LOG` environment variable to set the
                    // output target.
                    //
                    // - A value of `stderr` causes the log to be printed to STDERR.
                    // - A value of `windbg` causes the log to be output to the Windows debugger.
                    // - Anything else is interpreted as a file name, which causes the log to be
                    //   written to that file instead.
                    //
                    // If `NICE_LOG` is not set, then a dynamic logging output target is used instead.
                    // On Windows this causes log messages to be sent to the Windows debugger when
                    // one is attached, then falls back to STDERR. All other platforms use STDERR.
                    .with_writer(nice_plug::log::writer_from_env())
                    // It is recommended to disable ansi color support so that logging to a file is
                    // more readable.
                    .with_ansi(false)
                    .finish(),
            )
            .is_ok(),
        )
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        create_egui_editor(
            self.editor_state.clone(),
            self.repaint_notifier.clone(),
            EguiNiceSettings::new().with_resize_hint(RESIZE_HINT),
            NiceAutoVocoderEditor::new(self.params.clone()),
        )
    }
}

impl ClapPlugin for NiceAutoVocoder {
    const CLAP_ID: &'static str = "pw.kitl.autovocoder-nice";
    const CLAP_DESCRIPTION: Option<&'static str> = None;
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> =
        Some("https://github.com/kitlith/autovocoder-nice/issues");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::MultiEffects,
        ClapFeature::PitchShifter,
    ];
}

impl Vst3Plugin for NiceAutoVocoder {
    const VST3_CLASS_ID: [u8; 16] = *b"AutovocoderNiceP";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::PitchShift];
}

nice_export_clap!(NiceAutoVocoder);
nice_export_vst3!(NiceAutoVocoder);

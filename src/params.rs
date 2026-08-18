use std::sync::{Arc, atomic::Ordering};

use crate::enums::*;
use atomig::{Atom, AtomLogic};
use bitflag_attr::bitflag;
use nice_plug::prelude::*;

#[derive(Params, Debug)]
pub struct AutoVocoderParams {
    #[id = "mode"]
    pub mode: EnumParam<CarrierMode>,
    #[id = "f_note"]
    pub fixed_note: IntParam,
    #[id = "scale"]
    pub scale: EnumParam<ScaleKind>,
    #[id = "s_root"]
    pub scale_root: EnumParam<ScaleRoot>,
    #[id = "drywet"]
    pub mix: FloatParam,
    #[id = "portam"]
    pub portamento: FloatParam,
    #[id = "clevel"]
    pub carrier_level: FloatParam,
    #[id = "igain"]
    pub input_gain: FloatParam,
    #[id = "ogain"]
    pub output_gain: FloatParam,
    #[id = "c_on"]
    pub comp_on: BoolParam,
    #[id = "c_thre"]
    pub comp_threshold: FloatParam,
    #[id = "chtype"]
    pub chord_type: EnumParam<ChordVoicing>,
    #[id = "p_algo"]
    pub pitch_algo: EnumParam<PitchAlgorithm>,
    #[id = "ruscar"]
    pub carrier_chorus_on: BoolParam,
    #[id = "rusout"]
    pub output_chorus_on: BoolParam,
    #[id = "rusrat"]
    pub chorus_rate: FloatParam,
    #[id = "rusdep"]
    pub chorus_depth: FloatParam,
    #[id = "rusmix"]
    pub chorus_mix: FloatParam,
    #[id = "tr_on"]
    pub trem_on: BoolParam,
    #[id = "trrate"]
    pub trem_rate: FloatParam,
    #[id = "tr_dep"]
    pub trem_depth: FloatParam,
    #[id = "trshap"]
    pub trem_shape: FloatParam,
    #[id = "trtarg"]
    pub trem_target: EnumParam<LfoTarget>,
    #[id = "predrv"]
    pub pre_drive_on: BoolParam,
    #[id = "po_drv"]
    pub post_drive_on: BoolParam,
    // 8 more!
    #[id = "drvmod"]
    pub drive_mode: EnumParam<DriveMode>,
    #[id = "drvamt"]
    pub drive_amount: FloatParam,
    #[id = "crushe"]
    pub crusher_on: BoolParam,
    #[id = "crushb"]
    pub crusher_bits: IntParam,
    #[id = "crushr"]
    pub crusher_rate: FloatParam,
    #[id = "sub_on"]
    pub sub_on: BoolParam,
    #[id = "sublvl"]
    pub sub_level: FloatParam,
}

#[bitflag(u32)]
#[derive(Clone, Copy, PartialEq, Atom, AtomLogic, Default)]
pub enum UpdateFlags {
    CarrierMode = 1 << 0,
    Scale = 1 << 1,
    DryWet = 1 << 2,
    Portamento = 1 << 3,
    CarrierLevel = 1 << 4,
    InputGain = 1 << 5,
    OutputGain = 1 << 6,
    CompressorEnabled = 1 << 7,
    CompressorThreshold = 1 << 8,
    PitchAlgorithm = 1 << 9,
    CarrierChorusEnabled = 1 << 10,
    OutputChorusEnabled = 1 << 11,
    ChorusRate = 1 << 12,
    ChorusDepth = 1 << 13,
    ChorusMix = 1 << 14,
    TremoloEnabled = 1 << 15,
    TremoloRate = 1 << 16,
    TremoloDepth = 1 << 17,
    TremoloShape = 1 << 18,
    TremoloTarget = 1 << 19,
    PreDriveEnabled = 1 << 20,
    PostDriveEnabled = 1 << 21,
    DriveMode = 1 << 22,
    DriveAmount = 1 << 23,
    CrusherEnabled = 1 << 24,
    CrusherBits = 1 << 25,
    CrusherRate = 1 << 26,
    SubOscillatorEnabled = 1 << 27,
    SubOscillatorLevel = 1 << 28,
}

// based on bitflag_match, but without early exit and without exact matching
macro_rules! bitflag_check {
    // Eat an optional `,` following a block match arm
    ($operation:expr, { $pattern:expr => { $($body:tt)* } , $($t:tt)+ }) => {
        bitflag_check!($operation, { $pattern => { $($body)* } $($t)+ })
    };
    // Expand a block match arm `A => { .. }`
    ($operation:expr, { $pattern:expr => { $($body:tt)* } $($t:tt)+ }) => {
        {
            if $operation.contains($pattern) {
                $($body)*
            }

            bitflag_check!($operation, { $($t)+ })
        }
    };
    // Expand an expression match arm `A => x,`
    ($operation:expr, { $pattern:expr => $body:expr , $($t:tt)+ }) => {
        {
            if $operation.contains($pattern) {
                $body;
            }

            bitflag_check!($operation, { $($t)+ })
        }
    };
    // Expand the default case
    ($operation:expr, { _ => $default:expr $(,)? }) => {
        $default
    }
}

impl UpdateFlags {
    pub fn update(self, av: &mut autovocoder_dsp::AutoVocoder, params: &AutoVocoderParams) {
        bitflag_check!(self, {
            UpdateFlags::CarrierMode => {
                use autovocoder_dsp::CarrierMode::*;
                let mode = match params.mode.value() {
                    CarrierMode::Mono => Mono,
                    CarrierMode::Chord => Chord(params.chord_type.value().into()),
                    CarrierMode::Fixed => Fixed {
                        midi: params.fixed_note.value() as u8,
                    },
                    CarrierMode::FixedChord => FixedChord {
                        midi: params.fixed_note.value() as u8,
                        voicing: params.chord_type.value().into(),
                    },
                };
                av.set_carrier_mode(mode);
            }
            UpdateFlags::Scale => {
                let scale_kind = params.scale.value().to_int();
                let scale_root = params.scale_root.value() as u8;
                av.set_scale(autovocoder_dsp::Scale::from_int(scale_kind, scale_root));
            }
            UpdateFlags::DryWet => {
                av.set_dry_wet(params.mix.value());
                tracing::info!("{:?} (value: {})", params.mix, params.mix.value());
            },
            UpdateFlags::Portamento => av.set_portamento_ms(params.portamento.value()),
            UpdateFlags::CarrierLevel => av.set_carrier_level(params.carrier_level.value()),
            UpdateFlags::InputGain => av.set_input_gain_db(params.input_gain.value()),
            UpdateFlags::OutputGain => av.set_output_gain_db(params.output_gain.value()),
            UpdateFlags::CompressorEnabled => av.set_compressor_enabled(params.comp_on.value()),
            UpdateFlags::CompressorThreshold => av.set_compressor_threshold_db(params.comp_threshold.value()),
            UpdateFlags::PitchAlgorithm => av.set_pitch_algorithm(params.pitch_algo.value().into()),
            UpdateFlags::CarrierChorusEnabled => av.set_carrier_chorus_enabled(params.carrier_chorus_on.value()),
            UpdateFlags::OutputChorusEnabled => av.set_output_chorus_enabled(params.output_chorus_on.value()),
            UpdateFlags::ChorusRate => av.set_chorus_rate_hz(params.chorus_rate.value()),
            UpdateFlags::ChorusDepth => av.set_chorus_depth(params.chorus_depth.value()),
            UpdateFlags::ChorusMix => av.set_chorus_mix(params.chorus_mix.value()),
            UpdateFlags::TremoloEnabled => av.set_tremolo_enabled(params.trem_on.value()),
            UpdateFlags::TremoloRate => av.set_tremolo_rate_hz(params.trem_rate.value()),
            UpdateFlags::TremoloDepth => av.set_tremolo_depth(params.trem_depth.value()),
            UpdateFlags::TremoloShape => av.set_tremolo_shape(params.trem_shape.value()),
            UpdateFlags::TremoloTarget => av.set_tremolo_target(params.trem_target.value().into()),
            UpdateFlags::PreDriveEnabled => av.set_pre_drive_enabled(params.pre_drive_on.value()),
            UpdateFlags::PostDriveEnabled => av.set_post_drive_enabled(params.post_drive_on.value()),
            UpdateFlags::DriveMode => av.set_drive_mode(params.drive_mode.value().into()),
            UpdateFlags::DriveAmount => av.set_drive_amount(params.drive_amount.value().into()),
            UpdateFlags::CrusherEnabled => av.set_crusher_enabled(params.crusher_on.value()),
            UpdateFlags::CrusherBits => av.set_crusher_bits(params.crusher_bits.value() as f32), // *sigh*
            UpdateFlags::CrusherRate => av.set_crusher_rate(params.crusher_rate.value()),
            UpdateFlags::SubOscillatorEnabled => av.set_sub_enabled(params.sub_on.value()),
            UpdateFlags::SubOscillatorLevel => av.set_sub_level(params.sub_level.value()),
            _ => {}
        })
    }
}

impl AutoVocoderParams {
    pub fn new(wrapper: Arc<atomig::Atomic<UpdateFlags>>) -> Self {
        macro_rules! cb {
            ($flag:path) => {{
                let wrap = wrapper.clone();
                Arc::new(move |_| {
                    wrap.fetch_or($flag, Ordering::SeqCst);
                })
            }};
        }

        Self {
            mode: EnumParam::new("Carrier Mode", CarrierMode::Mono)
                .with_callback(cb!(UpdateFlags::CarrierMode)),
            fixed_note: IntParam::new(
                "Fixed Note (MIDI)",
                48,
                IntRange::Linear { min: 24, max: 84 },
            )
            .with_callback(cb!(UpdateFlags::CarrierMode)),
            scale: EnumParam::new("Scale", ScaleKind::Chromatic)
                .with_callback(cb!(UpdateFlags::Scale)),
            scale_root: EnumParam::new("Scale Root", ScaleRoot::C)
                .with_callback(cb!(UpdateFlags::Scale)),
            mix: FloatParam::new("Dry/Wet", 1., FloatRange::Linear { min: 0., max: 1. })
                .with_callback(cb!(UpdateFlags::DryWet)),
            portamento: FloatParam::new(
                "Portamento",
                25.,
                FloatRange::Linear { min: 1., max: 25. },
            )
            .with_callback(cb!(UpdateFlags::Portamento))
            .with_unit(" ms"),
            carrier_level: FloatParam::new(
                "Carrier Level",
                0.6,
                FloatRange::Linear { min: 0., max: 2. },
            )
            .with_callback(cb!(UpdateFlags::CarrierLevel)),
            input_gain: FloatParam::new(
                "Input Gain",
                6.,
                FloatRange::Linear {
                    min: -20.,
                    max: 60.,
                },
            )
            .with_callback(cb!(UpdateFlags::InputGain))
            .with_unit(" dB"),
            output_gain: FloatParam::new(
                "Output Gain",
                6.,
                FloatRange::Linear {
                    min: -20.,
                    max: 60.,
                },
            )
            .with_callback(cb!(UpdateFlags::OutputGain))
            .with_unit(" dB"),
            comp_on: BoolParam::new("Compressor", true)
                .with_callback(cb!(UpdateFlags::CompressorEnabled)),
            comp_threshold: FloatParam::new(
                "Compressor Threshold",
                -18.,
                FloatRange::Linear { min: -40., max: 0. },
            )
            .with_callback(cb!(UpdateFlags::CompressorThreshold))
            .with_unit(" dB"),
            chord_type: EnumParam::new("Chord Type", ChordVoicing::Major)
                .with_callback(cb!(UpdateFlags::CarrierMode)),
            pitch_algo: EnumParam::new("Pitch Detector", PitchAlgorithm::YinFft)
                .with_callback(cb!(UpdateFlags::PitchAlgorithm)),
            carrier_chorus_on: BoolParam::new("Chorus on Carrier", false)
                .with_callback(cb!(UpdateFlags::CarrierChorusEnabled)),
            output_chorus_on: BoolParam::new("Chorus on Output", false)
                .with_callback(cb!(UpdateFlags::OutputChorusEnabled)),
            chorus_rate: FloatParam::new(
                "Chorus Rate",
                0.7,
                FloatRange::Linear { min: 0.05, max: 5. },
            )
            .with_callback(cb!(UpdateFlags::ChorusRate))
            .with_unit(" hz"),
            chorus_depth: FloatParam::new(
                "Chorus Depth",
                0.5,
                FloatRange::Linear { min: 0., max: 1. },
            )
            .with_callback(cb!(UpdateFlags::ChorusDepth)),
            chorus_mix: FloatParam::new("Chorus Mix", 0.5, FloatRange::Linear { min: 0., max: 1. })
                .with_callback(cb!(UpdateFlags::ChorusMix)),
            trem_on: BoolParam::new("Tremolo", false)
                .with_callback(cb!(UpdateFlags::TremoloEnabled)),
            trem_rate: FloatParam::new(
                "Tremolo Rate",
                5.,
                FloatRange::Linear { min: 0.1, max: 20. },
            )
            .with_callback(cb!(UpdateFlags::TremoloRate)),
            trem_depth: FloatParam::new(
                "Tremolo Depth",
                0.7,
                FloatRange::Linear { min: 0., max: 1. },
            )
            .with_callback(cb!(UpdateFlags::TremoloDepth)),
            trem_shape: FloatParam::new(
                "Tremolo Shape (Sine→Square)",
                0.,
                FloatRange::Linear { min: 0., max: 1. },
            )
            .with_callback(cb!(UpdateFlags::TremoloShape)),
            trem_target: EnumParam::new("Mod LFO Target", LfoTarget::Amplitude)
                .with_callback(cb!(UpdateFlags::TremoloTarget)),
            pre_drive_on: BoolParam::new("Drive on Modulator (pre-vocoder)", false)
                .with_callback(cb!(UpdateFlags::PreDriveEnabled)),
            post_drive_on: BoolParam::new("Drive on Output (post)", false)
                .with_callback(cb!(UpdateFlags::PostDriveEnabled)),
            drive_mode: EnumParam::new("Drive Mode", DriveMode::Tape)
                .with_callback(cb!(UpdateFlags::DriveMode)),
            drive_amount: FloatParam::new(
                "Drive Amount",
                0.5,
                FloatRange::Linear { min: 0., max: 1. },
            )
            .with_callback(cb!(UpdateFlags::DriveAmount)),
            crusher_on: BoolParam::new("Bit Crusher", false)
                .with_callback(cb!(UpdateFlags::CrusherEnabled)),
            crusher_bits: IntParam::new(
                "Crusher Bit Depth",
                16,
                IntRange::Linear { min: 1, max: 16 },
            )
            .with_callback(cb!(UpdateFlags::CrusherBits)),
            crusher_rate: FloatParam::new(
                "Crusher Rate",
                1.,
                FloatRange::Linear { min: 0., max: 1. },
            )
            .with_callback(cb!(UpdateFlags::CrusherRate)),
            sub_on: BoolParam::new("Sub Oscillator", false)
                .with_callback(cb!(UpdateFlags::SubOscillatorEnabled)),
            sub_level: FloatParam::new("Sub Level", 0.5, FloatRange::Linear { min: 0., max: 1. })
                .with_callback(cb!(UpdateFlags::SubOscillatorLevel)),
        }
    }
}

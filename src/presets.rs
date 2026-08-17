use nice_plug::context::gui::ParamSetter;

use crate::{params::AutoVocoderParams, enums::*};

// based on <https://lukaswirth.dev/tlborm/decl-macros/building-blocks/parsing.html#struct>
macro_rules! impl_preset {
    // Named-Struct
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),*
        $(,)? }
    ) => {
        $( #[$meta] )*
        $vis struct $name {
            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_ty
            ),*
        }

        impl $name {
            pub fn apply(&self, params: &AutoVocoderParams, setter: &ParamSetter) {
                $(
                    setter.begin_set_parameter(&params.$field_name);
                    setter.set_parameter(&params.$field_name, self.$field_name);
                    setter.end_set_parameter(&params.$field_name);
                )*
            }
        }
    }
}

impl_preset! {
    #[derive(Debug)]
    pub struct AutoVocoderPreset {
        pub carrier_mode: CarrierMode,
        pub fixed_note: i32,
        pub scale_kind: ScaleKind,
        pub scale_root: ScaleRoot,
        pub dry_wet: f32,
        pub portamento: f32,
        pub carrier_level: f32,
        pub input_gain: f32,
        pub output_gain: f32,
        pub comp_on: bool,
        pub comp_threshold: f32,
        pub chord_type: ChordVoicing,
        pub pitch_algo: PitchAlgorithm,
        pub carrier_chorus_on: bool,
        pub output_chorus_on: bool,
        pub chorus_rate: f32,
        pub chorus_depth: f32,
        pub chorus_mix: f32,
        pub tremolo_on: bool,
        pub tremolo_rate: f32,
        pub tremolo_depth: f32,
        pub tremolo_shape: f32,
        pub tremolo_target: LfoTarget,
        pub pre_drive_on: bool,
        pub post_drive_on: bool,
        pub drive_mode: DriveMode,
        pub drive_amount: f32,
        pub crusher_on: bool,
        pub crusher_bits: i32,
        pub crusher_rate: f32,
        pub sub_on: bool,
        pub sub_level: f32,
    }
}

pub const PRESETS: [(&'static str, AutoVocoderPreset); 1] = [
    ("Chromatic Voice", AutoVocoderPreset {
        carrier_mode: CarrierMode::Mono,
        fixed_note: 48,
        scale_kind: ScaleKind::Chromatic,
        scale_root: ScaleRoot::C,
        dry_wet: 0.95,
        portamento: 1.0,
        carrier_level: 0.6,
        input_gain: 9.0,
        output_gain: 6.0,
        comp_on: true,
        comp_threshold: -18.,
        chord_type: ChordVoicing::Major,
        pitch_algo: PitchAlgorithm::YinFft,
        carrier_chorus_on: false,
        output_chorus_on: false,
        chorus_rate: 0.7,
        chorus_depth: 0.5,
        chorus_mix: 0.5,
        tremolo_on: false,
        tremolo_rate: 5.0,
        tremolo_depth: 0.7,
        tremolo_shape: 0.,
        tremolo_target: LfoTarget::Amplitude,
        pre_drive_on: false,
        post_drive_on: false,
        drive_mode: DriveMode::Tape,
        drive_amount: 0.5,
        crusher_on: false,
        crusher_bits: 16,
        crusher_rate: 1.,
        sub_on: false,
        sub_level: 0.5
    })
];

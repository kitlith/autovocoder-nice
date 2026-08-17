use nice_plug::context::gui::ParamSetter;

use crate::{enums::*, params::AutoVocoderParams};

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
        pub mode: CarrierMode,
        pub fixed_note: i32,
        pub scale: ScaleKind,
        pub scale_root: ScaleRoot,
        pub mix: f32,
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
        pub trem_on: bool,
        pub trem_rate: f32,
        pub trem_depth: f32,
        pub trem_shape: f32,
        pub trem_target: LfoTarget,
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

include!(concat!(env!("OUT_DIR"), "/presets.rs"));

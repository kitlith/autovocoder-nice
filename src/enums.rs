use nice_plug::prelude::Enum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ChordVoicing {
    #[name = "Power (5)"]
    Power, // 1 - 5            (0, 7)
    Major,      // 1 - 3 - 5        (0, 4, 7)
    Minor,      // 1 - b3 - 5       (0, 3, 7)
    Sus2,       // 1 - 2 - 5        (0, 2, 7)
    Sus4,       // 1 - 4 - 5        (0, 5, 7)
    Diminished, // 1 - b3 - b5      (0, 3, 6)
    Augmented,  // 1 - 3 - #5       (0, 4, 8)
    #[name = "Major 7"]
    Maj7, // 1 - 3 - 5 - 7    (0, 4, 7, 11)
    #[name = "Minor 7"]
    Min7, // 1 - b3 - 5 - b7  (0, 3, 7, 10)
    #[name = "Dominant 7"]
    Dom7, // 1 - 3 - 5 - b7   (0, 4, 7, 10)
    #[name = "Diminished 7"]
    Dim7, // 1 - b3 - b5 - bb7 (0, 3, 6, 9)
    #[name = "Half-Diminished"]
    HalfDim7, // 1 - b3 - b5 - b7 (0, 3, 6, 10)
    #[name = "Add 9"]
    Add9, // 1 - 3 - 5 - 9    (0, 4, 7, 14)
    #[name = "Dominant 9"]
    Dom9, // 1 - 3 - 5 - b7 - 9 (0, 4, 7, 10, 14)
    #[name = "Minor 9"]
    Min9, // 1 - b3 - 5 - b7 - 9 (0, 3, 7, 10, 14)
}

impl From<ChordVoicing> for autovocoder_dsp::ChordVoicing {
    fn from(value: ChordVoicing) -> Self {
        use autovocoder_dsp::ChordVoicing::*;
        match value {
            ChordVoicing::Power => Power,
            ChordVoicing::Major => Major,
            ChordVoicing::Minor => Minor,
            ChordVoicing::Sus2 => Sus2,
            ChordVoicing::Sus4 => Sus4,
            ChordVoicing::Diminished => Diminished,
            ChordVoicing::Augmented => Augmented,
            ChordVoicing::Maj7 => Maj7,
            ChordVoicing::Min7 => Min7,
            ChordVoicing::Dom7 => Dom7,
            ChordVoicing::Dim7 => Dim7,
            ChordVoicing::HalfDim7 => HalfDim7,
            ChordVoicing::Add9 => Add9,
            ChordVoicing::Dom9 => Dom9,
            ChordVoicing::Min9 => Min9,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum DriveMode {
    Tube,
    Tape,
    Fuzz,
}

impl From<DriveMode> for autovocoder_dsp::DriveMode {
    fn from(value: DriveMode) -> Self {
        use autovocoder_dsp::DriveMode::*;
        match value {
            DriveMode::Tube => Tube,
            DriveMode::Tape => Tape,
            DriveMode::Fuzz => Fuzz,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum LfoTarget {
    #[name = "Amplitude (tremolo)"]
    Amplitude,
    #[name = "Pitch (vibrato)"]
    Pitch,
    #[name = "Dry/Wet"]
    DryWet,
    #[name = "Carrier Level"]
    CarrierLevel,
}

impl From<LfoTarget> for autovocoder_dsp::LfoTarget {
    fn from(value: LfoTarget) -> Self {
        use autovocoder_dsp::LfoTarget::*;
        match value {
            LfoTarget::Amplitude => Amplitude,
            LfoTarget::Pitch => Pitch,
            LfoTarget::DryWet => DryWet,
            LfoTarget::CarrierLevel => CarrierLevel,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum PitchAlgorithm {
    #[name = "YIN (classic)"]
    YinClassic,
    #[name = "YIN (FFT)"]
    YinFft,
    #[name = "FFT (HPS)"]
    FftPeak,
}

impl From<PitchAlgorithm> for autovocoder_dsp::PitchAlgorithm {
    fn from(value: PitchAlgorithm) -> Self {
        use autovocoder_dsp::PitchAlgorithm::*;
        match value {
            PitchAlgorithm::YinClassic => YinClassic,
            PitchAlgorithm::YinFft => YinFft,
            PitchAlgorithm::FftPeak => FftPeak,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ScaleKind {
    Chromatic,
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    #[name = "Harmonic Minor"]
    HarmonicMinor,
    #[name = "Major Pentatonic"]
    MajorPentatonic,
    #[name = "Minor Pentatonic"]
    MinorPentatonic,
    Blues,
}

impl ScaleKind {
    pub fn to_int(self) -> i32 {
        use ScaleKind::*;
        match self {
            Chromatic => 0,
            Major => 1,
            Minor => 2,
            Dorian => 3,
            Phrygian => 4,
            Lydian => 5,
            Mixolydian => 6,
            HarmonicMinor => 7,
            MajorPentatonic => 8,
            MinorPentatonic => 9,
            Blues => 10,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ScaleRoot {
    C,
    #[name = "C#"]
    CSharp,
    D,
    #[name = "D#"]
    DSharp,
    E,
    F,
    #[name = "F#"]
    FSharp,
    G,
    #[name = "G#"]
    GSharp,
    A,
    #[name = "A#"]
    ASharp,
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum CarrierMode {
    Mono,
    #[name = "Chord (tracks)"]
    Chord,
    #[name = "Fixed Note"]
    Fixed,
    #[name = "Fixed Chord"]
    FixedChord,
}

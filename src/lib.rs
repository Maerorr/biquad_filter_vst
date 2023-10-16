use chorus::Chorus;
use filter::FilterType;
use nih_plug::prelude::*;
use std::{sync::{Arc, mpsc::channel}, collections::VecDeque, env};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

mod delay;
mod lfo;
mod editor;
mod chorus;
mod filter;

struct MaerorChorus {
    params: Arc<MaerorFilterParams>,
    sample_rate: f32,
    filter: filter::BiquadFilter,
    prev_filter_type : filter::FilterType,
}

#[derive(Params)]
struct MaerorFilterParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "FilterType"]
    filter_type: EnumParam<filter::FilterType>,

    #[id = "Cutoff"]
    cutoff: FloatParam,

    #[id = "Resonance"]
    resonance: FloatParam,

    #[id = "Gain"]
    gain: FloatParam,
}

impl Default for MaerorChorus {
    fn default() -> Self {
        Self {
            params: Arc::new(MaerorFilterParams::default()),
            sample_rate: 44100.0,
            filter: filter::BiquadFilter::new(),
            prev_filter_type: filter::FilterType::LowPass1,
        }
    }
}

impl Default for MaerorFilterParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            filter_type: EnumParam::new("Filter Type", filter::FilterType::LowPass1),

            // cutoff parameter in Hz, from 20 to 20k
            cutoff: FloatParam::new("Cutoff", 5000.0, FloatRange::Skewed { min: 20.0, max: 20000.0, factor: 0.5 } )
            .with_unit("")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(1))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            // resonance parameter from 0 to 30
            resonance: FloatParam::new("Resonance", 0.707, FloatRange::Linear { min: 0.5, max: 30.0 })
            .with_unit("")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // gain parameter from -30dB to 30dB
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for MaerorChorus {
    const NAME: &'static str = "maeror_filter";
    const VENDOR: &'static str = "236587 & 236598";
    const URL: &'static str = "none";
    const EMAIL: &'static str = "none";
    const VERSION: &'static str = "test";

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = _buffer_config.sample_rate as f32;

        self.filter.set_sample_rate(self.sample_rate);
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {

        // In current configuration this function iterates as follows:
        // 1. outer loop iterates block-size times
        // 2. inner loop iterates channel-size times. 

        for (i, channel_samples) in buffer.iter_samples().enumerate() {
            // Smoothing is optionally built into the parameters themselves
            // let gain = self.params.gain.smoothed.next();
            let filter_type = self.params.filter_type.value();
            let cutoff = self.params.cutoff.smoothed.next();
            let mut resonance = self.params.resonance.smoothed.next();
            let gain = self.params.gain.smoothed.next();

            if filter_type != self.prev_filter_type {
                self.prev_filter_type = filter_type;
                self.filter.reset_filter();
            }

            if filter_type == FilterType::SecondOrderAllPass {
                resonance = resonance.clamp(1.0, 1000.0)
            }

            self.filter.coefficients(filter_type, cutoff, resonance, gain);

            for (num, sample) in channel_samples.into_iter().enumerate() {
                if num == 0 {
                    *sample = self.filter.process_left(*sample)
                } else {
                    *sample = self.filter.process_right(*sample)
                }
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }
}

impl ClapPlugin for MaerorChorus {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for MaerorChorus {
    const VST3_CLASS_ID: [u8; 16] = *b"MaerorChorsRvdH.";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Filter];
}

//nih_export_clap!(MaerorChorus);
nih_export_vst3!(MaerorChorus);

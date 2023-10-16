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
mod comb;
mod delayingallpass;
mod reverb;

const MAX_BLOCK_SIZE: usize = 64;

struct ReverbPlugin {
    params: Arc<ReverbPluginParams>,
    comb_reverb: reverb::Reverb,
    schroeder_reverb: reverb::Reverb,
    lpf_comb_reverb: reverb::Reverb,
    lpf_schroeder_reverb: reverb::Reverb,
    sample_rate: f32,
}

// struct ScratchBuffer {
//     cutoff: [f32; MAX_BLOCK_SIZE],
//     resonance: [f32; MAX_BLOCK_SIZE],
//     gain: [f32; MAX_BLOCK_SIZE],
// }

// impl Default for ScratchBuffer {
//     fn default() -> Self {
//         Self {
//             cutoff: [0.0; MAX_BLOCK_SIZE],
//             resonance: [0.0; MAX_BLOCK_SIZE],
//             gain: [0.0; MAX_BLOCK_SIZE],
//         }
//     }
// }

#[derive(Params)]
struct ReverbPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "reverb-type"]
    reverb_type: EnumParam<reverb::ReverbType>,

    #[id = "decay"]
    decay: FloatParam,

    #[id = "damping"]
    damping: FloatParam,

    #[id = "comb type"]
    comb_type: EnumParam<comb::CombType>,

    #[id = "wet"]
    wet: FloatParam,

    #[id = "dry"]
    dry: FloatParam,
}

impl Default for ReverbPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ReverbPluginParams::default()),
            sample_rate: 44100.0,
            comb_reverb: reverb::Reverb::new(
                44100.0,
                100.0,
                reverb::ReverbType::Comb,
                0.0,
            ),
            schroeder_reverb: reverb::Reverb::new(
                44100.0,
                100.0,
                reverb::ReverbType::Schroeder,
                0.0,
            ),
            lpf_comb_reverb: reverb::Reverb::new(
                44100.0,
                100.0,
                reverb::ReverbType::Comb,
                0.2,
            ),
            lpf_schroeder_reverb: reverb::Reverb::new(
                44100.0,
                100.0,
                reverb::ReverbType::Schroeder,
                0.2,
            ),
        }
    }
}

impl Default for ReverbPluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            reverb_type: EnumParam::new("Reverb Type", reverb::ReverbType::Comb),

            decay: FloatParam::new("Decay", 250.0, FloatRange::Skewed { min: 100.0, max: 20000.0, factor: 0.3 })
            .with_unit("ms")
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            damping: FloatParam::new("Damping", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            comb_type: EnumParam::new("Comb Type", comb::CombType::Positive),

            wet: FloatParam::new("Wet", 0.25, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            dry: FloatParam::new("Dry", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for ReverbPlugin {
    const NAME: &'static str = "tsk reverb";
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
        self.comb_reverb.resize_buffers(self.sample_rate);
        self.schroeder_reverb.resize_buffers(self.sample_rate);
        self.lpf_comb_reverb.resize_buffers(self.sample_rate);
        self.lpf_schroeder_reverb.resize_buffers(self.sample_rate);
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
            let reverb_type = self.params.reverb_type.value();
            let comb_type = self.params.comb_type.value();
            let decay = self.params.decay.smoothed.next();
            let damping = self.params.damping.smoothed.next();
            let wet = self.params.wet.smoothed.next();
            let dry = self.params.dry.smoothed.next();

            match reverb_type {
                reverb::ReverbType::Comb => {
                    self.comb_reverb.set_params_comb(decay, comb_type);
                },
                reverb::ReverbType::Schroeder => {
                    self.schroeder_reverb.set_params_schroeder(decay, damping, comb_type)
                },
                reverb::ReverbType::LpfComb => {
                    self.lpf_comb_reverb.set_params_lpfcomb(decay, damping, comb_type);
                },
                reverb::ReverbType::Moorer => {
                    self.lpf_schroeder_reverb.set_params_moorer(decay, damping, comb_type)
                },
            };

            for (num, sample) in channel_samples.into_iter().enumerate() {
                if num == 0 {
                    match reverb_type {
                        reverb::ReverbType::Comb => {
                            *sample = *sample * dry + wet * self.comb_reverb.process_left(*sample)
                        },
                        reverb::ReverbType::Schroeder => {
                            *sample = *sample * dry + wet * self.schroeder_reverb.process_left(*sample)
                        },
                        reverb::ReverbType::LpfComb => {
                            *sample = *sample * dry + wet * self.lpf_comb_reverb.process_left(*sample)
                        },
                        reverb::ReverbType::Moorer => {
                            *sample = *sample * dry + wet * self.lpf_schroeder_reverb.process_left(*sample)
                        },
                    }
                } else {
                    match reverb_type {
                        reverb::ReverbType::Comb => {
                            *sample = *sample * dry + wet * self.comb_reverb.process_right(*sample)
                        },
                        reverb::ReverbType::Schroeder => {
                            *sample = *sample * dry + wet * self.schroeder_reverb.process_right(*sample)
                        },
                        reverb::ReverbType::LpfComb => {
                            *sample = *sample * dry + wet * self.lpf_comb_reverb.process_right(*sample)
                        },
                        reverb::ReverbType::Moorer => {
                            *sample = *sample * dry + wet * self.lpf_schroeder_reverb.process_right(*sample)
                        },
                    }
                }
                
                if dry + wet > 1.0 {
                    *sample = *sample / (dry + wet);
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

impl ClapPlugin for ReverbPlugin {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for ReverbPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"tsk__ReverbRvdH.";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Reverb];
}

//nih_export_clap!(MaerorChorus);
nih_export_vst3!(ReverbPlugin);

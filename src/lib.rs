use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use nih_plug::prelude::*;
use vizia_plug::ViziaState;
use nih_plug::prelude::SmoothingStyle::Linear;

mod editor;
mod gui;

#[derive(Params)]
pub struct MidiInterpolatorParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    // Interpolate between A and B
    #[id = "interpolate_a_b"]
    pub interpolate_a_b: FloatParam,

    pub channel_a: Arc<AtomicUsize>,

    pub channel_b: Arc<AtomicUsize>,
    //pub channel_b: EnumParam<MidiChannel>,
}

impl Default for MidiInterpolatorParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            interpolate_a_b: FloatParam::new(
                "Interpolate between Input 1 and 2",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
                .with_smoother(Linear(50.0)),

            //channel_a: EnumParam::new("Channel A", MidiChannel::Channel1),
            channel_a: Arc::new(AtomicUsize::new(1)),

            channel_b: Arc::new(AtomicUsize::new(2)),
        }
    }
}

struct MidiInterpolator {
    params: Arc<MidiInterpolatorParams>,
}

impl Default for MidiInterpolator {
    fn default() -> Self {
        let default_params = Arc::new(MidiInterpolatorParams::default());
        Self {
            params: default_params.clone(),
        }
    }
}

impl Plugin for MidiInterpolator {
    const NAME: &'static str = "MidiInterpolator";
    const VENDOR: &'static str = "Leon Focker";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "contact@leonfocker.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
    ];
    
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {

        // handle all incoming events
        while let Some(event) = context.next_event() {

            match event {
                NoteEvent::NoteOn {..} => {
                },
                NoteEvent::NoteOff {..} => {

                },
                _ => context.send_event(event),
            }
        }
        
        ProcessStatus::Normal
    }
}

impl ClapPlugin for MidiInterpolator {
    const CLAP_ID: &'static str = "leonfocker.midiinterpolator";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple distortion plugin flipping one bit of every sample");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for MidiInterpolator {
    const VST3_CLASS_ID: [u8; 16] = *b"MidiInterpolator";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(MidiInterpolator);
nih_export_vst3!(MidiInterpolator);

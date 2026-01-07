use std::collections::HashMap;
use std::ops::Rem;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
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

#[derive(Default)]
struct NoteAverage {
    cnt_a: u8,
    cnt_b: u8,
    note_a: u8,
    note_b: u8,
    velo_a: f32,
    velo_b: f32,
}

impl NoteAverage {
    fn return_event(&mut self, interp: f32, timing: u32, channel: u8) -> Option<PluginNoteEvent<MidiInterpolator>> {
        if self.cnt_a > 0 || self.cnt_b > 0 {
            if self.cnt_a > 0 {
                self.velo_a = self.velo_a / self.cnt_a as f32;
                self.note_a = self.note_a / self.cnt_a;
            } else {
                self.note_a = self.note_b / self.cnt_b;
            }
            if self.cnt_b > 0 {
                self.velo_b = self.velo_b / self.cnt_b as f32;
                self.note_b = self.note_b / self.cnt_b;
            } else {
                self.note_b = self.note_a;
            }

            let new_note = self.note_a as f32 * (1.0 - interp) + self.note_b as f32 * interp;
            let new_velo = self.velo_a * (1.0 - interp) + self.velo_b * interp;
            //dbg!(new_velo);
            //dbg!(new_note);

            // reset tmps
            self.cnt_a = 0;
            self.note_a = 0;
            self.velo_a = 0.0;
            self.cnt_b = 0;
            self.note_b = 0;
            self.velo_b = 0.0;

            // return average Note
            Some(NoteEvent::NoteOn {
                timing,
                voice_id: None,
                channel,
                note: new_note.round() as u8,
                velocity: new_velo,
            })
        } else { None }
    }

    fn advance_a(&mut self, note: u8, velocity: f32) -> () {
        self.cnt_a += 1;
        self.note_a += note;
        self.velo_a += velocity;
    }

    fn advance_b(&mut self, note: u8, velocity: f32) -> () {
        self.cnt_b += 1;
        self.note_b += note;
        self.velo_b += velocity;
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
        // TODO this doesn't handle durations (noteoffs), NoteOffs are sent even if the corresponding On wasn't
        // TODO change channel of output to note that has a Noteoff! Else notes sustend

        let interp = self.params.interpolate_a_b.value(); // TODO get this here?
        let chan_a = self.params.channel_a.load(SeqCst) - 1;
        let chan_b = self.params.channel_b.load(SeqCst) - 1;
        let mut note_average = NoteAverage::default();
        let mut last_timing = 0;

        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing,
                    channel,
                    note,
                    velocity,
                    ..
                } => {
                    // If this note is not at the same time as the last, return last notes average
                    if timing > last_timing {
                       if let Some(event) = note_average.return_event(interp, timing, channel) {
                           context.send_event(event);
                       }
                    }
                    last_timing = timing;

                    // Increase Average by this note
                    if channel as usize == chan_a {
                        note_average.advance_a(note, velocity);
                    } else if channel as usize == chan_b {
                        note_average.advance_b(note, velocity);
                    } else {
                        context.send_event(event);
                    }
                },
                _ => context.send_event(event),
            }
        }

        // get the last event out if necessary
        if let Some(event) = note_average.return_event(interp, last_timing, chan_a as u8) {
            context.send_event(event);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MidiInterpolator {
    const CLAP_ID: &'static str = "leonfocker.midiinterpolator";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("");
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

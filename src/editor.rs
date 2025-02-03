use nih_plug::prelude::{Editor};
use vizia_plug::vizia::prelude::*;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use vizia_plug::vizia::style::FontWeightKeyword::Bold;
use crate::gui::param_slider_knob::{ParamSliderKnob};
use crate::MidiInterpolatorParams;

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub(crate) params: Arc<MidiInterpolatorParams>,
    pub(crate) channels: (usize, usize),
}

impl Model for Data {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|my_event, _meta| match my_event {
            AppEvent::SetChannelA(selector) => {
                self.channels.0 = *selector;
                self.params.channel_a.store(*selector, SeqCst)
            }
            AppEvent::SetChannelB(selector) => {
                self.channels.1 = *selector;
                self.params.channel_b.store(*selector, SeqCst)
            }
        });
    }
}

enum AppEvent {
    SetChannelA(usize),
    SetChannelB(usize),
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 100))
}

pub(crate) fn create(
    params: Arc<MidiInterpolatorParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        
        Data {
            params: params.clone(),
            channels: (params.channel_a.load(SeqCst), params.channel_b.load(SeqCst)),
        }
            .build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Midi Interpolator")
                .font_weight(Bold)
                .font_size(25.0);

            HStack::new(cx, |cx| {
                dropdown_channel_selector(cx, false);

                //Element::new(cx).width(Pixels(10.0));

                ParamSliderKnob::new(cx, Data::params, |params| {
                    &params.interpolate_a_b
                })
                    .width(Pixels(100.0));

                //Element::new(cx).width(Pixels(10.0));

                dropdown_channel_selector(cx, true);
            })
                .alignment(Alignment::Center);

        })
            .alignment(Alignment::TopCenter);
    })
}

fn dropdown_channel_selector(cx: &mut Context, use_b: bool) {
    Dropdown::new(
        cx,
        move |cx| {
                Binding::new(cx, Data::channels,move |cx, channels| {
                    let channel = if use_b { channels.get(cx).1 } else {channels.get(cx).0 };
                    Label::new(cx, format!("Channel {}", channel ))
                        .alignment(Alignment::Center)
                        .on_press(move |cx| cx.emit(PopupEvent::Open));
                });
        },
        move |cx| {
            ScrollView::new(cx, move|cx| {
                for i in 1..=16 {
                    Label::new(cx, i)
                        .on_press(move |cx| {
                            if use_b {
                                cx.emit(AppEvent::SetChannelB(i));
                            } else {
                                cx.emit(AppEvent::SetChannelA(i));
                            }
                            cx.emit(PopupEvent::Close);
                        })
                        .width(Stretch(1.0));
                }
            })
                .show_horizontal_scrollbar(false)
                .show_vertical_scrollbar(false)
                .width(Stretch(1.0))
                .height(Pixels(60.0));
        },
    )
        .height(Stretch(1.0))
        .alignment(Alignment::Center)
        .on_press(move |cx| cx.emit(PopupEvent::Open));
}
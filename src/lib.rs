#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use serde::{Deserialize, Serialize};

use baseplug::{Plugin, ProcessContext, WindowOpenResult, UIModel, Model, UIFloatParam};
use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use raw_window_handle::HasRawWindowHandle;

use egui::CtxRef;
use egui_baseview::{EguiWindow, Queue, RenderSettings, Settings};

mod dsp;
use dsp::{Allpass, DegradedDelay};

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct ReverbModel {
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "g")]
        g: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "damping")]
        damping: f32,
        #[model(min = 0.0, max = 29.0)]
        #[parameter(name = "degrade_intensity")]
        degrade_intensity: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "degrade_speed")]
        degrade_speed: f32,
    }
}

impl Default for ReverbModel {
    fn default() -> Self {
        Self {
            g: 0.7,
            damping: 0.0,
            degrade_intensity: 0.0,
            degrade_speed: 0.0,
        }
    }
}

struct Reverb {
    allpass_one_l: Allpass,
    allpass_two_l: Allpass,
    allpass_three_l: Allpass,
    delay_one_l: DegradedDelay,
    delay_two_l: DegradedDelay,
    delay_three_l: DegradedDelay,
    delay_four_l: DegradedDelay,
    allpass_one_r: Allpass,
    allpass_two_r: Allpass,
    allpass_three_r: Allpass,
    delay_one_r: DegradedDelay,
    delay_two_r: DegradedDelay,
    delay_three_r: DegradedDelay,
    delay_four_r: DegradedDelay,
}

impl Plugin for Reverb {
    const NAME: &'static str = "Reverb";
    const PRODUCT: &'static str = "PISSYWISSY";
    const VENDOR: &'static str = "audiodog301";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = ReverbModel;

    #[inline]
    fn new(sample_rate: f32, _model: &ReverbModel) -> Self {
        Self {
            allpass_one_l: Allpass::new(sample_rate, 4410, 0.7),
            allpass_two_l: Allpass::new(sample_rate, 2999, -0.7),
            allpass_three_l: Allpass::new(sample_rate, 2646, 0.7),
            delay_one_l: DegradedDelay::new(sample_rate, 1323, 0.7),
            delay_two_l: DegradedDelay::new(sample_rate, 1499, 0.7),
            delay_three_l: DegradedDelay::new(sample_rate, 1676, 0.7),
            delay_four_l: DegradedDelay::new(sample_rate, 1852, 0.7),
            allpass_one_r: Allpass::new(sample_rate, 4410, 0.7),
            allpass_two_r: Allpass::new(sample_rate, 2999, -0.7),
            allpass_three_r: Allpass::new(sample_rate, 2646, 0.7),
            delay_one_r: DegradedDelay::new(sample_rate, 1323, 0.7),
            delay_two_r: DegradedDelay::new(sample_rate, 1499, 0.7),
            delay_three_r: DegradedDelay::new(sample_rate, 1676, 0.7),
            delay_four_r: DegradedDelay::new(sample_rate, 1852, 0.7),
        }
    }

    #[inline]
    fn process(&mut self, model: &ReverbModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            self.allpass_one_l.set_g(model.g[i]);
            self.allpass_two_l.set_g(-model.g[i]);
            self.allpass_three_l.set_g(model.g[i]);

            self.delay_one_l.set_feedback(model.g[i]);
            self.delay_two_l.set_feedback(model.g[i]);
            self.delay_three_l.set_feedback(model.g[i]);
            self.delay_four_l.set_feedback(model.g[i]);

            self.delay_one_l.set_feedback(1.0 - model.damping[i]);
            self.delay_two_l.set_feedback(1.0 - model.damping[i]);
            self.delay_three_l.set_feedback(1.0 - model.damping[i]);
            self.delay_four_l.set_feedback(1.0 - model.damping[i]);

            self.delay_one_l.set_amt(model.degrade_intensity[i] as i32);
            self.delay_two_l.set_amt(model.degrade_intensity[i] as i32);
            self.delay_three_l.set_amt(model.degrade_intensity[i] as i32);
            self.delay_four_l.set_amt(model.degrade_intensity[i] as i32);

            self.delay_one_l.set_ratio(model.degrade_speed[i]);
            self.delay_two_l.set_ratio(model.degrade_speed[i]);
            self.delay_three_l.set_ratio(model.degrade_speed[i]);
            self.delay_four_l.set_ratio(model.degrade_speed[i]);

            self.allpass_one_r.set_g(model.g[i]);
            self.allpass_two_r.set_g(-model.g[i]);
            self.allpass_three_r.set_g(model.g[i]);

            self.delay_one_r.set_feedback(model.g[i]);
            self.delay_two_r.set_feedback(model.g[i]);
            self.delay_three_r.set_feedback(model.g[i]);
            self.delay_four_r.set_feedback(model.g[i]);

            self.delay_one_r.set_feedback(1.0 - model.damping[i]);
            self.delay_two_r.set_feedback(1.0 - model.damping[i]);
            self.delay_three_r.set_feedback(1.0 - model.damping[i]);
            self.delay_four_r.set_feedback(1.0 - model.damping[i]);

            self.delay_one_r.set_amt(model.degrade_intensity[i] as i32);
            self.delay_two_r.set_amt(model.degrade_intensity[i] as i32);
            self.delay_three_r.set_amt(model.degrade_intensity[i] as i32);
            self.delay_four_r.set_amt(model.degrade_intensity[i] as i32);

            self.delay_one_r.set_ratio(model.degrade_speed[i]);
            self.delay_two_r.set_ratio(model.degrade_speed[i]);
            self.delay_three_r.set_ratio(model.degrade_speed[i]);
            self.delay_four_r.set_ratio(model.degrade_speed[i]);

            let delays_summed_l = (self.delay_one_l.process_sample(input[0][i])
                + self.delay_two_l.process_sample(input[0][i])
                + self.delay_three_l.process_sample(input[0][i])
                + self.delay_four_l.process_sample(input[0][i]))
                / 2.0;
            let delays_summed_r = (self.delay_one_r.process_sample(input[0][i])
                + self.delay_two_r.process_sample(input[0][i])
                + self.delay_three_r.process_sample(input[0][i])
                + self.delay_four_r.process_sample(input[0][i]))
                / 2.0;
            output[0][i] = ((self.allpass_three_l.process_sample(
                self.allpass_two_l
                    .process_sample(self.allpass_one_l.process_sample(delays_summed_l)),
            )) * model.g[i])
                + input[0][i];
            
                output[1][i] = ((self.allpass_three_r.process_sample(
                    self.allpass_two_r
                        .process_sample(self.allpass_one_r.process_sample(delays_summed_r)),
                )) * model.g[i])
                    + input[1][i];
        }
    }
}

impl baseplug::PluginUI for Reverb {
    type Handle = ();

    fn ui_size() -> (i16, i16) {
        (500, 300)
    }

    fn ui_open(parent: &impl HasRawWindowHandle, model: <Self::Model as Model<Self>>::UI) -> WindowOpenResult<Self::Handle> {
        let settings = Settings {
            window: WindowOpenOptions {
                title: String::from("egui-baseplug-examples gain"),
                size: Size::new(Self::ui_size().0 as f64, Self::ui_size().1 as f64),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            render_settings: RenderSettings::default(),
        };

        let state = State::new(model);

        EguiWindow::open_parented(
            parent,
            settings,
            state,
            // Called once before the first frame. Allows you to do setup code and to
            // call `ctx.set_fonts()`. Optional.
            |_egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut State| {},
            // Called before each frame. Here you should update the state of your
            // application and build the UI.
            |egui_ctx: &CtxRef, _queue: &mut Queue, state: &mut State| {
                // Must be called on the top of each frame in order to sync values from the rt thread.
                state.model.poll_updates();

                let format_value = |value_text: &mut String, param: &UIFloatParam<_, _>| {
                    *value_text = format!("{:.1} {}", param.unit_value(), param.unit_label());
                };

                let update_value_text = |value_text: &mut String, param: &UIFloatParam<_, _>| {
                    if param.updated_by_host() {
                        format_value(value_text, param)
                    }
                };

                let param_slider = |ui: &mut egui::Ui, label: &str, value_text: &mut String, param: &mut UIFloatParam<_, _>| {
                    ui.label(label);
                    ui.label(value_text.as_str());

                    // Use the normalized value of the param so we can take advantage of baseplug's value curves.
                    //
                    // You could opt to use your own custom widget if you wish, as long as it can operate with
                    // a normalized range from [0.0, 1.0].
                    let mut normal = param.normalized();
                    if ui.add(egui::Slider::new(&mut normal, 0.0..=1.0)).changed() {
                        param.set_from_normalized(normal);
                        format_value(value_text, param);
                    };
                };

                // Sync text values if there was automation.
                update_value_text(&mut state.g_value, &state.model.g);
                update_value_text(&mut state.damping_value, &state.model.damping);
                update_value_text(&mut state.degrade_intensity_value, &state.model.degrade_intensity);
                update_value_text(&mut state.degrade_speed_value, &state.model.degrade_speed);

                egui::Window::new("egui-baseplug gain demo").show(&egui_ctx, |ui| {
                    param_slider(ui, "sort of length", &mut state.g_value, &mut state.model.g);
                    param_slider(ui, "damping", &mut state.damping_value, &mut state.model.damping);
                    param_slider(ui, "degradation intensity", &mut state.degrade_intensity_value, &mut state.model.degrade_intensity);
                    param_slider(ui, "Gain Right", &mut state.degrade_speed_value, &mut state.model.degrade_speed);
                });

                // TODO: Add a way for egui-baseview to send a closure that runs every frame without always
                // repainting.
                egui_ctx.request_repaint();
            },
        );

        Ok(())
    }

    fn ui_close(mut handle: Self::Handle) {
        // TODO: Close window once baseview gets the ability to do this.
    }
}

struct State {
    model: ReverbModelUI<Reverb>,

    g_value: String,
    damping_value: String,
    degrade_intensity_value: String,
    degrade_speed_value: String,
}

impl State {
    pub fn new(model: ReverbModelUI<Reverb>) -> State {
        State {
            model,
            g_value: String::new(),
            damping_value: String::new(),
            degrade_intensity_value: String::new(),
            degrade_speed_value: String::new(),
        }
    }
}


baseplug::vst2!(Reverb, b"rvrb");
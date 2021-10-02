#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use serde::{Deserialize, Serialize};

use baseplug::{Plugin, ProcessContext};

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
    allpass_one: Allpass,
    allpass_two: Allpass,
    allpass_three: Allpass,
    delay_one: DegradedDelay,
    delay_two: DegradedDelay,
    delay_three: DegradedDelay,
    delay_four: DegradedDelay,
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
            allpass_one: Allpass::new(sample_rate, 4410, 0.7),
            allpass_two: Allpass::new(sample_rate, 2999, -0.7),
            allpass_three: Allpass::new(sample_rate, 2646, 0.7),
            delay_one: DegradedDelay::new(sample_rate, 1323, 0.7),
            delay_two: DegradedDelay::new(sample_rate, 1499, 0.7),
            delay_three: DegradedDelay::new(sample_rate, 1676, 0.7),
            delay_four: DegradedDelay::new(sample_rate, 1852, 0.7),
        }
    }

    #[inline]
    fn process(&mut self, model: &ReverbModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            self.allpass_one.set_g(model.g[i]);
            self.allpass_two.set_g(-model.g[i]);
            self.allpass_three.set_g(model.g[i]);

            self.delay_one.set_feedback(model.g[i]);
            self.delay_two.set_feedback(model.g[i]);
            self.delay_three.set_feedback(model.g[i]);
            self.delay_four.set_feedback(model.g[i]);

            self.delay_one.set_feedback(1.0 - model.damping[i]);
            self.delay_two.set_feedback(1.0 - model.damping[i]);
            self.delay_three.set_feedback(1.0 - model.damping[i]);
            self.delay_four.set_feedback(1.0 - model.damping[i]);

            self.delay_one.set_amt(model.degrade_intensity[i] as i32);
            self.delay_two.set_amt(model.degrade_intensity[i] as i32);
            self.delay_three.set_amt(model.degrade_intensity[i] as i32);
            self.delay_four.set_amt(model.degrade_intensity[i] as i32);

            self.delay_one.set_ratio(model.degrade_speed[i]);
            self.delay_two.set_ratio(model.degrade_speed[i]);
            self.delay_three.set_ratio(model.degrade_speed[i]);
            self.delay_four.set_ratio(model.degrade_speed[i]);

            let delays_summed = (self.delay_one.process_sample(input[0][i])
                + self.delay_two.process_sample(input[0][i])
                + self.delay_three.process_sample(input[0][i])
                + self.delay_four.process_sample(input[0][i]))
                / 2.0;
            output[0][i] = ((self.allpass_three.process_sample(
                self.allpass_two
                    .process_sample(self.allpass_one.process_sample(delays_summed)),
            )) * model.g[i])
                + input[0][i];
            output[1][i] = input[1][i];
        }
    }
}

baseplug::vst2!(Reverb, b"rvrb");

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use serde::{Serialize, Deserialize};

use baseplug::{
    ProcessContext,
    Plugin,
};

mod dsp;
use dsp::Allpass;

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct ReverbModel {
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "g")]
        g: f32,
    }
}

impl Default for ReverbModel {
    fn default() -> Self {
        Self {
            g: 0.7,
        }
    }
}

struct Reverb {
    allpass_one: Allpass,
    allpass_two: Allpass,
    allpass_three: Allpass,
    allpass_four: Allpass,
    allpass_five: Allpass,
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
            allpass_four: Allpass::new(sample_rate, 869, 0.7),
            allpass_five: Allpass::new(sample_rate, 258, 0.7),
        }
    }

    #[inline]
    fn process(&mut self, model: &ReverbModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;
	
	    for i in 0..ctx.nframes {
            //self.allpass.set_g(model.g[i]);
            
            output[0][i] = self.allpass_five.process_sample(self.allpass_four.process_sample(self.allpass_three.process_sample(self.allpass_two.process_sample(self.allpass_one.process_sample(input[0][i])))));
            output[1][i] = input[1][i];
        }
    }            
}

baseplug::vst2!(Reverb, b"rvrb");
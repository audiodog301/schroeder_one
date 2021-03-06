pub struct Downsampler {
    amt: i32,
    count: i32,
    stored_sample: f32,
}

impl Downsampler {
    pub fn new(amt: i32) -> Self {
        Self {
            amt,
            count: 0,
            stored_sample: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32, ratio: f32) -> f32 {
        self.count = (self.count + 2) % (self.amt + 1);
        if self.count == 0 {
            self.stored_sample = input;
        }

        (ratio * self.stored_sample) + ((1.0 - ratio) * input)
    }

    pub fn set_amt(&mut self, amt: i32) {
        self.amt = amt;
    }
}

pub struct Delay {
    buffer: Vec<f32>,
    input_i: usize,
    output_i: usize,
    former_output: f32,
}

impl Delay {
    pub fn new(sample_rate: f32, time: i32) -> Self {
        let length: usize = (sample_rate / 4.0) as usize;
        Self {
            buffer: vec![0.0; length],
            input_i: 0,
            output_i: (-time).rem_euclid(length as i32) as usize,
            former_output: 0.0,
        }
    }

    pub fn set_time(&mut self, time: i32) {
        self.output_i = (-time as i32).rem_euclid(self.buffer.len() as i32) as usize;
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.buffer[self.input_i] = input;
        self.former_output = self.buffer[self.output_i];

        self.input_i = (self.input_i + 1).rem_euclid(self.buffer.len());
        self.output_i = (self.output_i + 1).rem_euclid(self.buffer.len());

        self.former_output
    }
}

pub struct DelayWithFeedback {
    delay: Delay,
    feedback_delay: Delay,
    feedback: f32,
    former: f32,          //former output of main delay
    former_feedback: f32, //variable name makes it not sound like this, but it's the former output of the feedback delay
}

impl DelayWithFeedback {
    pub fn new(sample_rate: f32, time: i32, feedback: f32) -> Self {
        Self {
            delay: Delay::new(sample_rate, time),
            feedback_delay: Delay::new(sample_rate, time),
            feedback,
            former: 0.0,
            former_feedback: 0.0,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }
    pub fn set_time(&mut self, time: i32) {
        self.delay.set_time(time);
        self.feedback_delay.set_time(time);
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.former = self
            .delay
            .process_sample(input + (self.feedback * self.former_feedback));
        self.former_feedback = self.feedback_delay.process_sample(self.former);

        self.former
    }
}

pub struct DegradedDelay {
    delay: Delay,
    feedback_delay: Delay,
    feedback: f32,
    former: f32,          //former output of main delay
    former_feedback: f32, //variable name makes it not sound like this, but it's the former output of the feedback delay
    a: f32,
    ratio: f32,
    downsampler: Downsampler,
}

impl DegradedDelay {
    pub fn new(sample_rate: f32, time: i32, feedback: f32) -> Self {
        Self {
            delay: Delay::new(sample_rate, time),
            feedback_delay: Delay::new(sample_rate, time),
            feedback,
            former: 0.0,
            former_feedback: 0.0,
            a: 1.0,
            ratio: 0.0,
            downsampler: Downsampler::new(0),
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }
    pub fn set_time(&mut self, time: i32) {
        self.delay.set_time(time);
        self.feedback_delay.set_time(time);
    }
    pub fn set_a(&mut self, a: f32) {
        self.a = a;
    }
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio;
    }
    pub fn set_amt(&mut self, amt: i32) {
        self.downsampler.set_amt(amt);
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.former = self
            .delay
            .process_sample(input + (self.feedback * self.former_feedback));
        self.former_feedback = self.downsampler.process_sample(
            ((1.0 - self.a) * self.former_feedback)
                + (self.feedback_delay.process_sample(self.former) * self.a),
            self.ratio,
        );

        self.former
    }
}

pub struct Allpass {
    delay: DelayWithFeedback,
    g: f32,
}

impl Allpass {
    pub fn new(sample_rate: f32, time: i32, g: f32) -> Self {
        Self {
            delay: DelayWithFeedback::new(sample_rate, time, g),
            g: g,
        }
    }

    pub fn set_time(&mut self, time: i32) {
        self.delay.set_time(time);
    }
    pub fn set_g(&mut self, g: f32) {
        self.g = g;
        self.delay.set_feedback(g);
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        (input * -self.g) + (self.delay.process_sample(input) * (1.0 - self.g.powi(2)))
    }
}

pub struct Lfo {
    freq: f32,
    phase: f32,
}

impl Lfo {
    pub fn new(freq: f32) -> Self {
        Self { freq, phase: 0.0 }
    }

    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        let output = (self.phase * std::f32::consts::TAU).powi(2).sin();
        self.phase = (self.phase + self.freq / sample_rate).fract();
        output
    }
}

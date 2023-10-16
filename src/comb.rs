use std::collections::VecDeque;

use nih_plug::prelude::Enum;

use crate::{delay::Delay};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CombType {
    Positive,
    Negative
}

impl Enum for CombType {
    fn variants() -> &'static [&'static str] {
        &["Positive", "Negative"]
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&["Positive", "Negative"])
    }

    fn to_index(self) -> usize {
        match self {
            CombType::Positive => 0,
            CombType::Negative => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => CombType::Positive,
            1 => CombType::Negative,
            _ => panic!("Invalid index for CombType"),
        }
    }
}

#[derive(Clone)]
pub struct CombFilter {
    sample_rate: f32,
    comb_type: CombType,
    delay_samples: usize,
    delay_ms: f32,
    feedback: f32,
    left_delay_module: Delay,
    right_delay_module: Delay,
    left_feedback_buffer: Box<VecDeque<f32>>,
    right_feedback_buffer: Box<VecDeque<f32>>,
    left_x_buffer: Box<VecDeque<f32>>,
    right_x_buffer: Box<VecDeque<f32>>,
    use_lpf: bool,
    lpf_g: f32,
}

impl CombFilter {
    pub fn new(sample_rate: f32, comb_type: CombType, delay_ms: f32, feedback: f32, use_lfp: bool) -> Self {
        let delay_samples: usize = ((delay_ms as f32 / 1000.0) * sample_rate).round() as usize;

        let mut left_x_buffer: Box<VecDeque<f32>> 
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        let mut right_x_buffer: Box<VecDeque<f32>>
            = Box::new(VecDeque::with_capacity(sample_rate as usize));

        let mut left_feedback_buffer: Box<VecDeque<f32>> 
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        let mut right_feedback_buffer: Box<VecDeque<f32>> 
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        for _ in 0..(sample_rate as usize) {
            left_feedback_buffer.push_front(0.0);
            right_feedback_buffer.push_front(0.0);
            left_x_buffer.push_front(0.0);
            right_x_buffer.push_front(0.0);
        }

        Self {
            sample_rate,
            delay_samples,
            comb_type,
            delay_ms,
            feedback,
            left_delay_module: Delay::new(sample_rate as usize, delay_samples, 0.0),
            right_delay_module: Delay::new(sample_rate as usize, delay_samples, 0.0),
            left_x_buffer,
            right_x_buffer,
            left_feedback_buffer,
            right_feedback_buffer,
            use_lpf: use_lfp,
            lpf_g: 0.0,
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.left_delay_module.resize_buffers(sample_rate as usize);
        self.right_delay_module.resize_buffers(sample_rate as usize);
        self.left_feedback_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_feedback_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.left_x_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_x_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
    
        for _ in 0..(sample_rate as usize) {
            self.left_feedback_buffer.push_front(0.0);
            self.right_feedback_buffer.push_front(0.0);
            self.left_x_buffer.push_front(0.0);
            self.right_x_buffer.push_front(0.0);
        }
    }

    pub fn set_damp(&mut self, damp: f32) {
        self.lpf_g = damp;
    }

    pub fn get_delay_ms(&self) -> f32 {
        self.delay_ms
    }

    pub fn set_params(&mut self, feedback: f32, use_lfp: bool, damp: f32, comb_type: CombType) {
        self.feedback = feedback;
        self.left_delay_module.delay = self.delay_samples;
        self.right_delay_module.delay = self.delay_samples;
        self.use_lpf = use_lfp;
        self.lpf_g = damp;
        self.comb_type = comb_type;
    }

    pub fn process_left(&mut self, x: f32) -> f32 {
        let mut y = x + self.feedback * self.left_feedback_buffer[self.delay_samples];

        match self.comb_type {
            CombType::Positive => {
                y += self.left_delay_module.process_sample(x, self.delay_samples);
            },
            CombType::Negative => {
                y -= self.left_delay_module.process_sample(x, self.delay_samples);
            }
        }

        // simple lpf
        if self.use_lpf {
            y -= self.lpf_g * self.left_x_buffer[self.delay_samples + 1];
            y += self.lpf_g * self.left_feedback_buffer[1];

            self.left_x_buffer.rotate_right(1);
            self.left_x_buffer[0] = x;
        }
        
        self.left_feedback_buffer.rotate_right(1);
        self.left_feedback_buffer[0] = y;
        y
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let mut y = x + self.feedback * self.right_feedback_buffer[self.delay_samples];

        match self.comb_type {
            CombType::Positive => {
                y += self.right_delay_module.process_sample(x, self.delay_samples);
            },
            CombType::Negative => {
                y -= self.right_delay_module.process_sample(x, self.delay_samples);
            }
        }

        // simple lpf
        if self.use_lpf {
            y -= self.lpf_g * self.right_x_buffer[self.delay_samples + 1];
            y += self.lpf_g * self.right_feedback_buffer[1];

            self.right_x_buffer.rotate_right(1);
            self.right_x_buffer[0] = x;
        }
        
        self.right_feedback_buffer.rotate_right(1);
        self.right_feedback_buffer[0] = y;
        y
    }
}
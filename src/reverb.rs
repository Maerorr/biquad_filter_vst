use nih_plug::prelude::Enum;
use rand::Rng;

use crate::{delayingallpass::DelayingAllPass, comb::{CombFilter, CombType}};


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReverbType {
    Comb,
    Schroeder,
    LpfComb,
    Moorer,
}

impl Enum for ReverbType {
    fn variants() -> &'static [&'static str] {
        &["Comb", "Schroeder", "Low-Pass Comb", "Moorer"]
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&["comb", "schroeder", "lpfcomb", "moorer"])
    }

    fn to_index(self) -> usize {
        match self {
            ReverbType::Comb => 0,
            ReverbType::Schroeder => 1,
            ReverbType::LpfComb => 2,
            ReverbType::Moorer => 3,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => ReverbType::Comb,
            1 => ReverbType::Schroeder,
            2 => ReverbType::LpfComb,
            3 => ReverbType::Moorer,
            _ => panic!("Invalid index for ReverbType"),
        }
    } 
}

#[derive(Clone)]
pub struct Reverb {
    left_combs: Vec<CombFilter>,
    right_combs: Vec<CombFilter>,
    left_allpasses: Vec<DelayingAllPass>,
    right_allpasses: Vec<DelayingAllPass>,
    decay: f32,
    reverb_type: ReverbType,
    sample_rate: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32, decay: f32, reverb_type: ReverbType, damp: f32) -> Self {
        let mut left_combs = Vec::new();
        let mut left_allpasses = Vec::new();
        let mut right_combs = Vec::new();
        let mut right_allpasses = Vec::new();

        let mut rng = rand::thread_rng();

        match reverb_type {
            ReverbType::Comb => {
                let delays_ms = [21.0, 26.0, 31.0, 37.0];
                for i in 0..4 {
                    // random delay between 3 and 50 ms
                    let ldelay_ms = delays_ms[i] + rng.gen_range(-0.5..0.5);
                    let ldelay_seconds = ldelay_ms / 1000.0;

                    let power = -(3.0 * ldelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    left_combs.push(CombFilter::new(sample_rate, comb_type, ldelay_ms, g, false));
                    left_combs[i].set_damp(0.0);

                    let rdelay_ms = delays_ms[i] + rng.gen_range(-0.5..0.5);
                    let rdelay_seconds = rdelay_ms / 1000.0;

                    let power = -(3.0 * rdelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    right_combs.push(CombFilter::new(sample_rate, comb_type, rdelay_ms, g, false));
                    right_combs[i].set_damp(0.0);
                }
            },
            ReverbType::Schroeder => {
                let mut ldelay_ms = 15.02;
                let mut rdelay_ms = 15.01;
                for i in 0..4 {
                    ldelay_ms = ldelay_ms * 1.5;
                    let ldelay_seconds = ldelay_ms / 1000.0;
                    let ldelay_samples = (ldelay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * ldelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    left_combs.push(CombFilter::new(sample_rate, comb_type, ldelay_ms, g, false));
                    left_combs[i].set_damp(damp);

                    rdelay_ms = rdelay_ms * 1.5;
                    let rdelay_seconds = rdelay_ms / 1000.0;
                    let rdelay_samples = (rdelay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * rdelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    right_combs.push(CombFilter::new(sample_rate, comb_type, rdelay_ms, g, false));
                    right_combs[i].set_damp(damp);
                }

                for _ in 0..4 {
                    // random delay between 1 and 5 ms
                    let delay = rng.gen_range(1.0..5.0);
                    left_allpasses.push(DelayingAllPass::new(sample_rate, delay, 0.707));

                    let delay = rng.gen_range(1.0..5.0);
                    right_allpasses.push(DelayingAllPass::new(sample_rate, delay, 0.707));
                }
            },
            ReverbType::LpfComb => {
                let mut ldelay_ms = 15.02;
                let mut rdelay_ms = 15.01;
                for i in 0..6 {
                    ldelay_ms = ldelay_ms * 1.5;
                    let ldelay_seconds = ldelay_ms / 1000.0;
                    let ldelay_samples = (ldelay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * ldelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    left_combs.push(CombFilter::new(sample_rate, comb_type, ldelay_ms, g, false));
                    left_combs[i].set_damp(damp);

                    rdelay_ms = rdelay_ms * 1.5;
                    let rdelay_seconds = rdelay_ms / 1000.0;
                    let rdelay_samples = (rdelay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * rdelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    right_combs.push(CombFilter::new(sample_rate, comb_type, rdelay_ms, g, false));
                    right_combs[i].set_damp(damp);
                }
            },
            ReverbType::Moorer => {
                let mut ldelay_ms = 15.02;
                let mut rdelay_ms = 15.01;
                for i in 0..6 {
                    ldelay_ms = ldelay_ms * 1.5;
                    let ldelay_seconds = ldelay_ms / 1000.0;
                    let ldelay_samples = (ldelay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * ldelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    left_combs.push(CombFilter::new(sample_rate, comb_type, ldelay_ms, g, false));
                    left_combs[i].set_damp(damp);

                    rdelay_ms = rdelay_ms * 1.5;
                    let rdelay_seconds = rdelay_ms / 1000.0;
                    let rdelay_samples = (rdelay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * rdelay_seconds as f32) / (decay) ;

                    let g = 10f32.powf(power);

                    let comb_type = CombType::Positive;
                    right_combs.push(CombFilter::new(sample_rate, comb_type, rdelay_ms, g, false));
                    right_combs[i].set_damp(damp);
                }

                for _ in 0..4 {
                    // random delay between 1 and 5 ms
                    let delay = rng.gen_range(1.0..5.0);
                    left_allpasses.push(DelayingAllPass::new(sample_rate, delay, 0.707));

                    let delay = rng.gen_range(1.0..5.0);
                    right_allpasses.push(DelayingAllPass::new(sample_rate, delay, 0.707));
                }
            },
        }

        Self {
            left_combs,
            right_combs,
            left_allpasses,
            right_allpasses,
            decay,
            reverb_type,
            sample_rate,
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for comb in self.left_combs.iter_mut() {
            comb.resize_buffers(sample_rate);
        }
        for comb in self.right_combs.iter_mut() {
            comb.resize_buffers(sample_rate);
        }
        for allpass in self.left_allpasses.iter_mut() {
            allpass.resize_buffers(sample_rate);
        }
        for allpass in self.right_allpasses.iter_mut() {
            allpass.resize_buffers(sample_rate);
        }
    }

    pub fn set_params_comb(&mut self, decay: f32, comb_type: CombType) {
        self.decay = decay;
        for comb in self.left_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            comb.set_params(  g, false, 0.0, comb_type)
        }
        for comb in self.right_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            comb.set_params( g, false, 0.0, comb_type)
        }
    }

    pub fn set_params_schroeder(&mut self, decay: f32, damp: f32, comb_type: CombType) {
        self.decay = decay;
        for comb in self.left_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            comb.set_params(g, false, damp, comb_type)
        }
        for comb in self.right_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            comb.set_params( g, false, damp, comb_type)
        }
    }

    pub fn set_params_lpfcomb(&mut self, decay: f32, damp: f32, comb_type: CombType) {
        self.decay = decay;
        for comb in self.left_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            let damp = damp.clamp(0.0, 0.9999);

            let new_g = g * (1.0 - damp);

            comb.set_params( new_g, true, damp, comb_type)
        }
        for comb in self.right_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            let damp = damp.clamp(0.0, 0.9999);

            let new_g = g * (1.0 - damp);

            comb.set_params( new_g, true, damp, comb_type)
        }
    }

    pub fn set_params_moorer(&mut self, decay: f32, damp: f32, comb_type: CombType) {
        self.decay = decay;
        for comb in self.left_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            let damp = damp.clamp(0.0, 0.9999);

            let new_g = g * (1.0 - damp);

            comb.set_params( new_g, true, damp, comb_type)
        }
        for comb in self.right_combs.iter_mut() {
            let power = -(3.0 * comb.get_delay_ms() / 1000.0 ) / (decay / 1000.0) ;

            let g = 10f32.powf(power);

            let damp = damp.clamp(0.0, 0.9999);

            let new_g = g * (1.0 - damp);

            comb.set_params( new_g, true, damp,comb_type)
        }
    }

    pub fn process_left(&mut self, x: f32) -> f32 {
        let mut y = 0.0;
        match self.reverb_type {
            ReverbType::Comb => {
                for comb in self.left_combs.iter_mut() {
                    y += comb.process_left(x);
                }
                y *= 0.25;
            },
            ReverbType::Schroeder => {
                for (i, comb) in self.left_combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_left(x);
                    } else {
                        y -= comb.process_left(x);
                    }
                }
                y *= 0.25;
                for allpass in self.left_allpasses.iter_mut() {
                    y = allpass.process_left(y);
                }
            },
            ReverbType::LpfComb => {
                for (i, comb) in self.left_combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_left(x);
                    } else {
                        y -= comb.process_left(x);
                    }
                }
                y /= 6.0;
            },
            ReverbType::Moorer => {
                for (i, comb) in self.left_combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_left(x);
                    } else {
                        y -= comb.process_left(x);
                    }
                }
                y /= 6.0;
                for allpass in self.left_allpasses.iter_mut() {
                    y = allpass.process_left(y);
                }
            },
        }
        y
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let mut y = 0.0;
        match self.reverb_type {
            ReverbType::Comb => {
                for comb in self.right_combs.iter_mut() {
                    y += comb.process_right(x);
                }
                y *= 0.25;
            },
            ReverbType::Schroeder => {
                for (i, comb) in self.right_combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_right(x);
                    } else {
                        y -= comb.process_right(x);
                    }
                }
                y *= 0.25;
                for allpass in self.right_allpasses.iter_mut() {
                    y = allpass.process_right(y);
                }
            },
            ReverbType::LpfComb => {
                for (i, comb) in self.right_combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_right(x);
                    } else {
                        y -= comb.process_right(x);
                    }
                }
                y /= 6.0;
            },
            ReverbType::Moorer => {
                for (i, comb) in self.right_combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_right(x);
                    } else {
                        y -= comb.process_right(x);
                    }
                }
                y /= 6.0;
                for allpass in self.right_allpasses.iter_mut() {
                    y = allpass.process_right(y);
                }
            },
        }
        y
    }
}
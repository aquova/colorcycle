use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

pub const DAY_SECS: usize = 86400;
pub const CYCLE_SPEED: usize = 280;

pub trait ColorCycle {
    fn get_size(&self) -> (u32, u32);
    fn get_indices(&self) -> &[usize];
    fn get_animated_indices(&self) -> &[usize];
    fn cycle(&mut self, time: Option<usize>, dt: usize) -> Vec<Color>;
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Cycle {
    pub reverse: usize,
    pub rate: usize,
    pub low: usize,
    pub high: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BaseData {
    pub width: u32,
    pub height: u32,
    pub colors: Vec<Color>,
    pub cycles: Vec<Cycle>,
    pub pixels: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct TimelineData {
    pub base: BaseData,
    pub palettes: HashMap<String, BaseData>,
    pub timeline: HashMap<String, String>,
}

pub fn palette_shift(base_colors: &mut [Color], low_idx: usize, high_idx: usize, amount: usize, rev: bool) {
    for _ in 0..amount {
        if rev {
            let low = base_colors[low_idx];
            for idx in low_idx..high_idx {
                base_colors[idx] = base_colors[idx + 1];
            }
            base_colors[high_idx] = low;
        } else {
            let high = base_colors[high_idx];
            for idx in (low_idx + 1..high_idx + 1).rev() {
                base_colors[idx] = base_colors[idx - 1];
            }
            base_colors[low_idx] = high;
        }
    }
}

pub fn calc_amount(rate: usize, dt: usize, cycle_num: usize, anim_style: usize) -> usize {
    // These values seem rather arbitrary
    let amount = match anim_style {
        0..=2 => {
            (dt * rate / 1000) % cycle_num
        },
        3 => {
            let mut  a = (dt * rate / 1000) % 2 * cycle_num;
            if a >= cycle_num {
                a = 2 * cycle_num - a;
            }
            a
        },
        _ => {
            let mut a = (dt * rate / 1000) % cycle_num;
            a = ((a as f64 * 2.0 * PI).sin() / cycle_num as f64 + 1.0) as usize;
            if anim_style == 4 {
                a *= cycle_num / 4;
            } else if anim_style == 5 {
                a *= cycle_num / 2;
            }

            a
        }
    };

    amount
}

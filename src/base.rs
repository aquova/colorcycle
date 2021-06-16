use crate::shared::*;
use std::fs::File;

pub struct StaticImage {
    width: u32,
    height: u32,
    colors: Vec<Color>,
    cycles: Vec<Cycle>,
    pixels: Vec<usize>,
    anim: Vec<usize>,
}

impl StaticImage {
    pub fn new(json_file: &str) -> Result<Self, serde_json::Error> {
        let file = File::open(json_file).expect("Unable to open file");
        let json: Result<BaseData, serde_json::Error> = serde_json::from_reader(&file);
        match json {
            Ok(data) => Ok(StaticImage::from_data(data)),
            Err(e) => Err(e)
        }
    }

    pub fn from_data(data: BaseData) -> Self {
        let mut anim = Vec::new();

        for cycle in data.cycles.iter() {
            if cycle.rate != 0 {
                for idx in cycle.low..=cycle.high {
                    anim.push(idx);
                }
            }
        }

        Self {
            width: data.width,
            height: data.height,
            colors: data.colors,
            cycles: data.cycles,
            pixels: data.pixels,
            anim: anim,
        }
    }

    pub fn get_palette(&self) -> &[Color] {
        &self.colors
    }

    pub fn get_cycles(&self) -> &[Cycle] {
        &self.cycles
    }
}

impl ColorCycle for StaticImage {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_indices(&self) -> &[usize] {
        &self.pixels
    }

    fn get_animated_indices(&self) -> &[usize] {
        &self.anim
    }

    fn cycle(&mut self, _time: Option<usize>, dt: usize) -> Vec<Color> {
        let mut palette = self.colors.to_owned().clone();

        for cycle in self.cycles.iter() {
            let num = cycle.high - cycle.low + 1;
            let rate = cycle.rate / CYCLE_SPEED;
            let amount = calc_amount(rate, dt, num, cycle.reverse);
            palette_shift(&mut palette, cycle.low, cycle.high, amount, cycle.reverse == 2);
        }

        palette
    }
}

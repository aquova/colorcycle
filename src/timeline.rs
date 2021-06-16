use crate::base::StaticImage;
use crate::shared::*;

use std::collections::HashMap;
use std::fs::File;

struct TodData {
    pub palette: Vec<Color>,
    pub offset: usize,
    pub size: usize,
    pub next: Option<Vec<Color>>,
}

pub struct TimelineImage {
    base: StaticImage,
    palettes: HashMap<String, BaseData>,
    timeline: HashMap<String, String>,
    tod: Option<TodData>,
}

impl TimelineImage {
    pub fn new(json_file: &str) -> Result<Self, serde_json::Error> {
        let file = File::open(json_file).expect("Unable to open file");
        let json: Result<TimelineData, serde_json::Error> = serde_json::from_reader(&file);
        match json {
            Ok(data) => {
                let s = Self {
                    base: StaticImage::from_data(data.base),
                    palettes: data.palettes,
                    timeline: data.timeline,
                    tod: None,
                };
                Ok(s)
            },
            Err(e) => Err(e)
        }
    }

    fn get_palette(&mut self, time: Option<usize>) -> &[Color] {
        if self.tod.is_none() {
            match time {
                Some(t) => {
                    let t = t % DAY_SECS;

                    let mut smallest_over = usize::MAX;
                    let mut end = 0;
                    let mut pal_name = String::new();

                    for (tod, name) in self.timeline.iter() {
                        let todi = tod.parse::<usize>().unwrap();
                        if todi < t {
                            let dt = t - todi;
                            if dt < smallest_over {
                                smallest_over = dt;
                                pal_name = name.to_owned().to_string();
                                end = todi;
                            }
                        }
                    }

                    // If we didn't find one, use the smallest
                    if smallest_over == usize::MAX {
                        for (tod, name) in self.timeline.iter() {
                            let todi = tod.parse::<usize>().unwrap();
                            if todi < smallest_over {
                                smallest_over = todi;
                                pal_name = name.to_owned().to_string();
                                end = todi;
                            }
                        }
                    }

                    let mut smallest_next = usize::MAX;
                    let mut next_name = String::new();
                    let mut next_end = 0;

                    for (tod, name) in self.timeline.iter() {
                        let todi = tod.parse::<usize>().unwrap();
                        if end < todi {
                            let dt = todi - end;
                            if dt < smallest_next {
                                smallest_next = dt;
                                next_name = name.to_owned().to_string();
                                next_end = todi;
                            }
                        }
                    }

                    // If we didn't find a next, wrap around and use the smallest
                    if smallest_next == usize::MAX {
                        for (tod, name) in self.timeline.iter() {
                            let todi = tod.parse::<usize>().unwrap();
                            if todi < smallest_next {
                                smallest_next = todi;
                                next_name = name.to_owned().to_string();
                                next_end = todi;
                            }
                        }
                    }

                    let pal = self.palettes[&pal_name].colors.to_owned();
                    let next_pal = self.palettes[&next_name].colors.to_owned();
                    self.tod = Some(TodData{ palette: pal, offset: smallest_over, size: next_end - end, next: Some(next_pal)});
                },
                None => {
                    let pal = self.base.get_palette().to_owned();
                    self.tod = Some(TodData{ palette: pal, offset: 1, size: 1, next: None });
                }
            }
        }

        &self.tod.as_ref().unwrap().palette
    }

    fn fade_color(&self, start: &Color, end: &Color, percent: f32) -> Color {
        let red = start.r as f32 + (end.r as f32 - start.r as f32) * percent;
        let green = start.g as f32 + (end.g as f32 - start.g as f32) * percent;
        let blue = start.b as f32 + (end.b as f32 - start.b as f32) * percent;

        Color { r: red as u8, g: green as u8, b: blue as u8 }
    }
}

impl ColorCycle for TimelineImage {
    fn get_size(&self) -> (u32, u32) {
        self.base.get_size()
    }

    fn get_indices(&self) -> &[usize] {
        self.base.get_indices()
    }

    fn get_animated_indices(&self) -> &[usize] {
        self.base.get_animated_indices()
    }

    fn cycle(&mut self, time: Option<usize>, dt: usize) -> Vec<Color> {
        let mut palette = self.get_palette(time).to_owned().clone();

        // Fade palette for time of day
        if let Some(data) = &self.tod {
            let percent = data.offset as f32 / data.size as f32;
            if let Some(next_pal) = &data.next {
                for idx in 0..palette.len() {
                    palette[idx] = self.fade_color(&palette[idx], &next_pal[idx], percent);
                }
            }
        }

        for cycle in self.base.get_cycles().iter() {
            let num = cycle.high - cycle.low + 1;
            let rate = cycle.rate / CYCLE_SPEED;
            let amount = calc_amount(rate, dt, num, cycle.reverse);
            palette_shift(&mut palette, cycle.low, cycle.high, amount, cycle.reverse == 2);
        }

        palette
    }
}

mod base;
mod shared;
mod timeline;

use crate::base::StaticImage;
use crate::shared::{DAY_SECS, ColorCycle};
use crate::timeline::TimelineImage;

use std::time::{Duration, Instant};
use std::thread::sleep;

use argparse::{ArgumentParser, Store};
use chrono::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE: u32 = 2;
const FRAME_TIME: u64 = 1000 / 60;

fn main() {
    let mut filename = String::new();
    let mut raw_time = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Display Living Worlds ColorCycle image");
        ap.refer(&mut filename).add_argument(&"filename", Store, "Input file").required();
        ap.refer(&mut raw_time).add_option(&["--time"], Store, "Set the time to display, if applicable (HH:MM, 24h time)");
        ap.parse_args_or_exit();
    }

    let mut img = match parse_input(&filename) {
        Ok(data) => data,
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    let time: usize = parse_time(&raw_time);
    let (width, height) = img.get_size();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Color Cycle", width * SCALE, height * SCALE)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let start_time = Instant::now();
    let mut first = true;

    'mainloop: loop {
        let loop_start = Instant::now();
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..}=> {
                    break 'mainloop;
                },
                _ => ()
            }
        }

        let dt = start_time.elapsed();
        draw(&mut img, Some(time + dt.as_secs() as usize), dt.as_millis() as usize, &mut canvas, first);
        first = false;
        let loop_dt = loop_start.elapsed().as_millis() as u64;
        if loop_dt < FRAME_TIME {
            let sleep_time = FRAME_TIME - loop_dt;
            sleep(Duration::from_millis(sleep_time));
        }
    }
}

fn parse_input(filename: &str) -> Result<Box<dyn ColorCycle>, &'static str> {
    let input = TimelineImage::new(filename);
    match input {
        Ok(img) => {
            return Ok(Box::new(img));
        },
        Err(_) => {
            let input = StaticImage::new(filename);
            if let Ok(timeline) = input {
                return Ok(Box::new(timeline));
            }
        }
    }

    Err("Unable to parse json file")
}

fn draw(img: &mut Box<dyn ColorCycle>, tod: Option<usize>, dt: usize, canvas: &mut Canvas<Window>, first_draw: bool) {
    if first_draw {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
    }

    let palette = img.cycle(tod, dt);
    let indices = img.get_indices();
    let anim = img.get_animated_indices();
    for (i, idx) in indices.iter().enumerate() {
        if first_draw || anim.contains(idx) {
            let pixel = palette[*idx];
            let pix_color = Color::RGB(pixel.r, pixel.g, pixel.b);
            canvas.set_draw_color(pix_color);
            let (width, _) = img.get_size();

            let x = (i as u32) % width;
            let y = (i as u32) / width;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn parse_time(t: &str) -> usize {
    let nums: Vec<_> = t.split(":").collect();
    if nums.len() == 2 {
        if let Ok(hour) = nums[0].parse::<usize>() {
            if let Ok(min) = nums[1].parse::<usize>() {
                let time = hour * 3600 + min * 60;
                return time % DAY_SECS;
            }
        }
    }

    eprintln!("Rendering at current time");
    get_current_time()
}

fn get_current_time() -> usize {
    let local = Local::now();
    let t = local.hour() * 3600 + local.minute() * 60 + local.second();
    t as usize
}

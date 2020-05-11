pub mod cpu;
pub mod display;
pub mod keyboard;

extern crate sdl2;

use std::collections::HashSet;
use std::env;
use std::time::{Duration, Instant};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub fn update_canvas(canvas: &mut Canvas<Window>, chip8_cpu: &cpu::CPU) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for i in 0..chip8_cpu.display.fb.len() {
        let x = i % 64;
        let y = i / 64;
        if chip8_cpu.display.get_pixel(x, y) {
            canvas
                .fill_rect(Rect::new((x * 12) as i32, (y * 12) as i32, 12, 12))
                .unwrap();
        }
    }
}

pub fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Expected path to Chip8 ROM as first argument, CPU speed in HZ as second argument");
    }

    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Rusty CHIP8", 768, 384)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let audio_device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        }
    })?;

    let mut canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    // Initialize chip8 CPU
    let mut chip8_cpu = cpu::CPU::new();
    chip8_cpu.reset();
    chip8_cpu.load_rom(&args[1]);

    // Calculate how often we need to run a cpu cycle
    const US_IN_S: u32 = 1000000;
    let exec_time = US_IN_S / args[2].parse::<u32>().unwrap();

    let mut cpu_exec_clk = Instant::now();
    let mut delay_timer_clk = Instant::now();
    let mut beep_timer = Instant::now();

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                _ => {}
            }
        }

        if beep_timer.elapsed().as_millis() > 20 {
            audio_device.pause();
        }

        // Create a set of pressed Keys.
        let keys: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        // Update the key state in the chip8 CPU
        // This is not optimal, make it a reference eventually
        chip8_cpu.keyboard.update_keys(keys.clone());

        if cpu_exec_clk.elapsed().as_micros() >= exec_time.into() {
            chip8_cpu.exec_cycle();
            cpu_exec_clk = Instant::now();
        }

        let mut output_beep = false;
        if delay_timer_clk.elapsed().as_micros() >= (US_IN_S / 60).into() {
            output_beep = chip8_cpu.update_timers();
            delay_timer_clk = Instant::now();
        }

        if output_beep {
            beep_timer = Instant::now();
            audio_device.resume();
        }

        if chip8_cpu.display.need_redraw {
            update_canvas(&mut canvas, &chip8_cpu);
            chip8_cpu.display.need_redraw = false;
        }

        canvas.present();
        ::std::thread::sleep(Duration::from_micros(100));
    }

    Ok(())
}
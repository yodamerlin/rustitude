use ggez::event::{EventHandler, KeyCode};
use ggez::graphics::{Color, DrawMode, DrawParam};
use ggez::input;
use ggez::nalgebra::Point2;
use ggez::*;

use std::collections::VecDeque;

const GAME_SPEED: f32 = 150.0;
const WAVE_FRONT_FREQUENCY: f32 = 1.0;
const WAVE_FRONT_AMPLITUDE: f32 = 70.0;
const WAVE_FRONT_AMPLITUDE_SMALL: f32 = 20.0;
const WAVE_RADIUS: f32 = 16.0;

const LIFE_RECOVER: f32 = 10.0;
const LIFE_DEPLETE: f32 = 20.0;
const LIFE_MAXIMUM: f32 = 100.0;

struct AmplitudeGameState<'a> {
    wave_front: Wave,
    wave_section: &'a mut VecDeque<WaveSection>,
    time: f32,
    life: f32,
}

struct WaveSection {
    x: f32,
    y: f32,
    color: Color,
}

struct Wave {
    x: f32,
    y: f32,
}

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Amplitude", "yodamerlin")
        .build()
        .expect("Could not create ggez context, death will now occur.");

    graphics::set_window_title(&ctx, "Amplitude");
    let screen_size = graphics::screen_coordinates(&ctx);

    let mut state = AmplitudeGameState {
        wave_front: Wave {
            x: screen_size.w / 8.0,
            y: screen_size.h / 2.0,
        },
        wave_section: &mut VecDeque::new(),
        time: 0.0,
        life: 100.0,
    };

    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

impl EventHandler for AmplitudeGameState<'_> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx);
        let dt = (dt.as_nanos() as f32) / 1_000_000_000.0;

        // update wave front
        let previous_sine_function =
            (self.time * 2.0 * std::f32::consts::PI * WAVE_FRONT_FREQUENCY).sin();
        let next_sine_function =
            ((self.time + dt) * 2.0 * std::f32::consts::PI * WAVE_FRONT_FREQUENCY).sin();

        let sine_function_difference = previous_sine_function - next_sine_function;

        // apply checks here to reduce amplitude

        let amplitude: f32;

        if input::keyboard::is_key_pressed(&ctx, KeyCode::Space) {
            if self.life > 0.0 {
                amplitude = WAVE_FRONT_AMPLITUDE_SMALL;
                self.life = self.life - LIFE_DEPLETE * dt;
            } else {
                amplitude = WAVE_FRONT_AMPLITUDE;
            }
        } else {
            amplitude = WAVE_FRONT_AMPLITUDE;
            self.life = self.life + LIFE_RECOVER * dt;
        }

        if self.life > LIFE_MAXIMUM {
            self.life = LIFE_MAXIMUM;
        }

        let section_color: Color;
        if amplitude == WAVE_FRONT_AMPLITUDE {
            section_color = Color::new(1.0, 0.0, 0.0, 1.0)
        } else {
            section_color = Color::new(0.0, 0.0, 1.0, 1.0)
        }

        self.wave_front.y = self.wave_front.y + sine_function_difference * amplitude;

        self.time = self.time + dt;

        // update wave back

        let new_wave_section = WaveSection {
            x: self.wave_front.x,
            y: self.wave_front.y,
            color: section_color,
        };
        for section in self.wave_section.iter_mut() {
            section.x = section.x - dt * GAME_SPEED
        }

        self.wave_section.push_back(new_wave_section);

        // remove elements that are behind the screen
        loop {
            if let Some(section) = self.wave_section.get(0) {
                if section.x < -32.0 {
                    self.wave_section.pop_front();
                    continue;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let mb = &mut graphics::MeshBuilder::new();

        let screen_size = graphics::screen_coordinates(&ctx);
        let width = screen_size.w;

        let mut bar_width = self.life / 100.0 * (width - 10.0);
        if bar_width < 0.0 {
            bar_width = 0.0;
        }

        mb.rectangle(
            DrawMode::fill(),
            graphics::Rect::new(5.0, 5.0, bar_width, 16.0),
            self.wave_section[self.wave_section.len() - 1].color, // I feel bad about this
        );

        for section in self.wave_section.iter() {
            mb.circle(
                DrawMode::fill(),
                Point2::new(section.x, section.y),
                WAVE_RADIUS,
                0.2,
                section.color,
            );
        }

        for s in mb.build(ctx) {
            graphics::draw(ctx, &s, DrawParam::new())?;
        }

        graphics::present(ctx)
    }
}

#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value
)]
use std::f64::consts::PI;

use nannou::color::hsl;
use nannou::{app, color::BLACK, event::Update, App, Frame};
use nannou_audio::{Buffer, Host, Stream};
use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    app(model).update(update).simple_window(view).run();
}

struct Model {
    numbers: Vec<i32>,
    n: usize,
    stream: Stream<(f64, f64)>,
    check_progress: usize,
}

const MAX: i32 = 1000;

fn model(_app: &App) -> Model {
    let mut numbers = (1..=MAX).collect::<Vec<_>>();
    numbers.shuffle(&mut thread_rng());
    let n = numbers.len();
    Model {
        numbers,
        n,
        stream: Host::new()
            .new_output_stream((0., 0.))
            .render(audio)
            .build()
            .unwrap(),
        check_progress: 0,
    }
}

fn audio((frequency, x): &mut (f64, f64), buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let square_amp = if (2. * PI * *x) % 1. < 0.5 { 1. } else { -1. };
        *x += *frequency / sample_rate;
        *x %= sample_rate;
        for channel in frame {
            *channel = square_amp * volume;
        }
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    const MAX_FREQUENCY: f64 = 440.;
    fn flip(numbers: &mut [i32], mut index: usize) {
        let mut left = 0;
        while left < index {
            numbers.swap(left, index);
            index -= 1;
            left += 1;
        }
    }
    if model.n == 0 {
        model.check_progress += 20;
        if model.check_progress > MAX as usize {
            model.stream.pause().unwrap();
        } else {
            let check_progress = model.check_progress;
            model
                .stream
                .send(move |(frequency, _)| {
                    *frequency = check_progress as f64 / MAX as f64 * MAX_FREQUENCY;
                })
                .unwrap();
        }
        return;
    }
    let maxdex = model
        .numbers
        .iter()
        .take(model.n)
        .enumerate()
        .max_by_key(|(_, number)| **number)
        .unwrap()
        .0;
    if maxdex != model.n - 1 {
        if maxdex != 0 {
            flip(&mut model.numbers, maxdex);
        }
        flip(&mut model.numbers, model.n - 1);
        model
            .stream
            .send(move |(frequency, _)| *frequency = maxdex as f64 / MAX as f64 * MAX_FREQUENCY)
            .unwrap();
    }
    model.n -= 1;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    for (index, number) in model.numbers.iter().enumerate() {
        draw.rect()
            .h(*number as f32 / MAX as f32 * app.window_rect().h())
            .w(app.window_rect().w() / model.numbers.len() as f32)
            .x(
                (index as f32 - model.numbers.len() as f32 / 2.) * app.window_rect().w()
                    / model.numbers.len() as f32
                    + app.window_rect().w() / model.numbers.len() as f32 / 2.,
            )
            .y(*number as f32 / MAX as f32 * app.window_rect().h() / 2.
                - app.window_rect().h() / 2.)
            .color(hsl((*number as f32 - 1.) / MAX as f32, 1., 0.5));
    }
    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();
}

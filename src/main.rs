#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use druid::piet::{Text, TextLayout, TextLayoutBuilder};
use druid::widget::{prelude::*, TextBox, ValueTextBox};
use druid::{
    kurbo::{Circle, CircleSegment, Line},
    text::{Formatter, Selection, Validation, ValidationError},
    widget::{Button, Flex, Painter, SizedBox},
    AppLauncher, Color, Data, Env, Widget, WindowDesc,
};
use druid::{Lens, WidgetExt};

use std::f64::consts::PI;
use std::fmt;
use std::str::FromStr;

const WINDOW_SIZE: f64 = 1400.;

#[derive(Clone, Data, Lens)]
struct Time {
    hours: u8,
    minutes: u8,
}

#[derive(Debug, Clone)]
struct InputValidationError {
    message: String,
}

impl InputValidationError {
    fn new(message: &str) -> Self {
        InputValidationError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for InputValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for InputValidationError {}

fn format(value: &u8) -> String {
    value.to_string()
}

fn validate_partial_input(input: &str, upper_limit: u8) -> Validation {
    if input.is_empty() {
        return Validation::success();
    }
    match u8::from_str(input) {
        Ok(num) if num <= upper_limit => Validation::success(),
        _ => Validation::failure(InputValidationError::new(
            "Input must be a number between 0 and 24.",
        )),
    }
}

fn value(input: &str) -> Result<u8, ValidationError> {
    if input.is_empty() {
        return Ok(0);
    }
    input.parse::<u8>().map_err(|_| {
        ValidationError::new(InputValidationError::new(
            "Input must be a number between 0 and 24.",
        ))
    })
}

struct HoursFormatter;
struct MinutesFormatter;

impl Formatter<u8> for HoursFormatter {
    fn format(&self, value: &u8) -> String {
        format(value)
    }

    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        validate_partial_input(input, 24)
    }

    fn value(&self, input: &str) -> Result<u8, ValidationError> {
        value(input)
    }
}

impl Formatter<u8> for MinutesFormatter {
    fn format(&self, value: &u8) -> String {
        format(value)
    }

    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        validate_partial_input(input, 60)
    }

    fn value(&self, input: &str) -> Result<u8, ValidationError> {
        value(input)
    }
}

fn decrease_hours(data: &mut Time) {
    if data.hours == 0 {
        data.hours = 23;
    } else {
        data.hours -= 1;
    }
}

fn increase_hours(data: &mut Time) {
    if data.hours == 23 {
        data.hours = 0;
    } else {
        data.hours += 1;
    }
}

fn decrease_minutes(data: &mut Time) {
    if data.minutes == 0 {
        data.minutes = 59;
        decrease_hours(data)
    } else {
        data.minutes -= 1;
    }
}

fn increase_minutes(data: &mut Time) {
    if data.minutes == 59 {
        data.minutes = 0;
        increase_hours(data)
    } else {
        data.minutes += 1;
    }
}

#[test]
fn decrease_hours_decreases_hours() {
    let mut time = Time {
        hours: 12,
        minutes: 0,
    };
    decrease_hours(&mut time);
    assert_eq!(time.hours, 11);
}

#[test]
fn decrease_hours_moves_to_23_from_0() {
    let mut time = Time {
        hours: 0,
        minutes: 0,
    };
    decrease_hours(&mut time);
    assert_eq!(time.hours, 23);
}

#[test]
fn increase_hours_decreases_hours() {
    let mut time = Time {
        hours: 12,
        minutes: 0,
    };
    increase_hours(&mut time);
    assert_eq!(time.hours, 13);
}

#[test]
fn increase_hours_moves_to_0_from_23() {
    let mut time = Time {
        hours: 23,
        minutes: 0,
    };
    increase_hours(&mut time);
    assert_eq!(time.hours, 0);
}

#[test]
fn increase_minutes_increases_minutes() {
    let mut time = Time {
        hours: 12,
        minutes: 30,
    };
    increase_minutes(&mut time);
    assert_eq!(time.hours, 12);
    assert_eq!(time.minutes, 31);
}

#[test]
fn increase_minutes_increases_minutes_and_hours() {
    let mut time = Time {
        hours: 12,
        minutes: 59,
    };
    increase_minutes(&mut time);
    assert_eq!(time.hours, 13);
    assert_eq!(time.minutes, 0);
}

#[test]
fn decrease_minutes_decreases_minutes() {
    let mut time = Time {
        hours: 12,
        minutes: 30,
    };
    decrease_minutes(&mut time);
    assert_eq!(time.hours, 12);
    assert_eq!(time.minutes, 29);
}

#[test]
fn decrease_minutes_decreases_minutes_and_hours() {
    let mut time = Time {
        hours: 12,
        minutes: 00,
    };
    decrease_minutes(&mut time);
    assert_eq!(time.hours, 11);
    assert_eq!(time.minutes, 59);
}

fn ui_builder() -> impl Widget<Time> {
    // Text fields with hours and minutes
    let valuetextbox_hours = ValueTextBox::new(TextBox::new(), HoursFormatter).lens(Time::hours);
    let valuetextbox_minutes =
        ValueTextBox::new(TextBox::new(), MinutesFormatter).lens(Time::minutes);

    // Buttons for increasing and decreasing hours and minutes
    let increment_hours =
        Button::new("+").on_click(|_ctx, data: &mut Time, _env| increase_hours(data));
    let decrement_hours =
        Button::new("-").on_click(|_ctx, data: &mut Time, _env| decrease_hours(data));
    let increment_minutes =
        Button::new("+").on_click(|_ctx, data: &mut Time, _env| increase_minutes(data));
    let decrement_minutes =
        Button::new("-").on_click(|_ctx, data: &mut Time, _env| decrease_minutes(data));

    // Clock graphics
    let clock = Painter::new(|ctx: &mut PaintCtx, data: &Time, _: &Env| {
        let boundaries = ctx.size().to_rect();
        let center = (boundaries.width() / 2.0, boundaries.height() / 2.0);
        let circle = Circle::new(center, center.0.min(center.1));
        ctx.fill(circle, &Color::WHITE);

        let orange = Color::rgb8(240, 128, 0);

        let colors: [Color; 12] = [
            Color::RED,
            orange,
            Color::YELLOW,
            Color::GREEN,
            Color::BLUE,
            Color::PURPLE,
            Color::RED,
            orange,
            Color::YELLOW,
            Color::GREEN,
            Color::BLUE,
            Color::PURPLE,
        ];

        for (n, color) in colors.iter().enumerate() {
            let circle_segment = CircleSegment::new(
                center,
                WINDOW_SIZE / 4. - WINDOW_SIZE / 40. * 2.,
                WINDOW_SIZE / 4. - WINDOW_SIZE / 100.,
                2. * PI / 12. * n as f64,
                2. * PI / 12.,
            );
            ctx.fill(circle_segment, color);
        }

        for n in 0..12 {
            let x = (n as f64 / 12. * 2. * PI).cos();
            let y = (n as f64 / 12. * 2. * PI).sin();
            ctx.stroke(
                Line::new(
                    (
                        x * (WINDOW_SIZE / 4. - WINDOW_SIZE / 40. * 2.) + (WINDOW_SIZE / 4.),
                        y * (WINDOW_SIZE / 4. - WINDOW_SIZE / 40. * 2.) + (WINDOW_SIZE / 4.),
                    ),
                    (
                        x * (WINDOW_SIZE / 4. - WINDOW_SIZE / 100.) + (WINDOW_SIZE / 4.),
                        y * (WINDOW_SIZE / 4. - WINDOW_SIZE / 100.) + (WINDOW_SIZE / 4.),
                    ),
                ),
                &Color::BLACK,
                WINDOW_SIZE / 100.,
            );
        }
        for n in 0..60 {
            let x = (n as f64 / 60. * 2. * PI).cos();
            let y = (n as f64 / 60. * 2. * PI).sin();
            ctx.stroke(
                Line::new(
                    (
                        x * (WINDOW_SIZE / 4. - WINDOW_SIZE / 40.) + (WINDOW_SIZE / 4.),
                        y * (WINDOW_SIZE / 4. - WINDOW_SIZE / 40.) + (WINDOW_SIZE / 4.),
                    ),
                    (
                        x * (WINDOW_SIZE / 4. - WINDOW_SIZE / 100.) + (WINDOW_SIZE / 4.),
                        y * (WINDOW_SIZE / 4. - WINDOW_SIZE / 100.) + (WINDOW_SIZE / 4.),
                    ),
                ),
                &Color::BLACK,
                WINDOW_SIZE / 200.,
            );
        }

        let minutes_x = (data.minutes as f64 / 60. * 2. * PI - PI / 2.).cos();
        let minutes_y = (data.minutes as f64 / 60. * 2. * PI - PI / 2.).sin();

        let hours_x = (((data.hours as f64) % 12. + data.minutes as f64 / 60.) / 12. * 2. * PI
            - PI / 2.)
            .cos();
        let hours_y = (((data.hours as f64) % 12. + data.minutes as f64 / 60.) / 12. * 2. * PI
            - PI / 2.)
            .sin();

        for n in 0..12 {
            let text_layout = ctx
                .text()
                .new_text_layout(format!("{}", n + 1))
                .font(druid::piet::FontFamily::SYSTEM_UI, WINDOW_SIZE * 0.03)
                .text_color(Color::BLACK)
                .build()
                .unwrap();

            let text_size = text_layout.size();

            let x = (n as f64 / 12. * 2. * PI - PI / 2. + 1. / 6. * PI).cos();
            let y = (n as f64 / 12. * 2. * PI - PI / 2. + 1. / 6. * PI).sin();
            let text_position = (
                x * (WINDOW_SIZE / 40. * 7.25) - text_size.width / 2. + (WINDOW_SIZE / 4.),
                y * (WINDOW_SIZE / 40. * 7.25) - text_size.height / 2. + (WINDOW_SIZE / 4.),
            );

            ctx.draw_text(&text_layout, text_position);
        }

        ctx.stroke(
            Line::new(
                (WINDOW_SIZE / 4., WINDOW_SIZE / 4.),
                (
                    minutes_x * (WINDOW_SIZE / 40. * 6.5) + (WINDOW_SIZE / 4.),
                    minutes_y * (WINDOW_SIZE / 40. * 6.5) + (WINDOW_SIZE / 4.),
                ),
            ),
            &Color::BLACK,
            WINDOW_SIZE / 100.,
        );
        ctx.stroke(
            Line::new(
                (WINDOW_SIZE / 4., WINDOW_SIZE / 4.),
                (
                    hours_x * (WINDOW_SIZE / 40. * 3.25) + (WINDOW_SIZE / 4.),
                    hours_y * (WINDOW_SIZE / 40. * 3.25) + (WINDOW_SIZE / 4.),
                ),
            ),
            &Color::BLACK,
            WINDOW_SIZE / 100.,
        );
    });

    // Creating a layout using the graphics described above
    Flex::column()
        .with_child(
            SizedBox::new(clock)
                .width(WINDOW_SIZE / 2.)
                .height(WINDOW_SIZE / 2.),
        )
        .with_spacer(4.0)
        .with_child(
            Flex::row()
                .with_child(
                    Flex::column()
                        .with_child(increment_hours)
                        .with_child(valuetextbox_hours)
                        .with_child(decrement_hours),
                )
                .with_child(
                    Flex::column()
                        .with_child(increment_minutes)
                        .with_child(valuetextbox_minutes)
                        .with_child(decrement_minutes),
                ),
        )
}

fn main() {
    let main_window = WindowDesc::new(ui_builder())
        .window_size((WINDOW_SIZE * 0.6, WINDOW_SIZE * 0.6))
        .title("Clock");
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(Time {
            hours: 12,
            minutes: 0,
        })
        .unwrap()
}

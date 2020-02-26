//! Generate a mandelbrot set and draw it to screen

#![no_std]
#![no_main]

#[allow(unused_imports)]
use panic_halt;
use pygamer as hal;

use embedded_graphics::drawable::Pixel;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::style::PrimitiveStyle;

use hal::adc::Adc;
use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::gclk::pchctrl::GEN_A::GCLK11;
use hal::pac::{CorePeripherals, Peripherals};
use itertools::Itertools;

use hal::gpio;

use smart_leds::hsv::{hsv2rgb, Hsv};

// The width and height of the display
const DISP_SIZE_X: i32 = 160;
const DISP_SIZE_Y: i32 = 128;

const KEYBOARD_DELTA: i32 = 4;
const CURSOR_COLOR: Rgb565 = RgbColor::RED;
const PIXEL_SIZE: i32 = 4;

fn idx_to_point(idx: usize) -> Point {
    Point::new(idx as i32 / 28, idx as i32 % 28)
}

fn point_to_idx(pair: (i32, i32)) -> usize {
    (pair.0 * 28 + pair.1) as usize
}

fn mnist_pixel(pair: (i32, i32), pixels: &[u32; 784]) -> Pixel<Rgb565> {
    let idx = point_to_idx(pair);
    match pixels[idx] {
        1 => Pixel(
            Point::new(pair.0 * PIXEL_SIZE, pair.1 * PIXEL_SIZE),
            RgbColor::WHITE,
        ),
        _ => Pixel(
            Point::new(pair.0 * PIXEL_SIZE, pair.1 * PIXEL_SIZE),
            RgbColor::BLACK,
        ),
    }
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT).split();
    let mut delay = hal::delay::Delay::new(core.SYST, &mut clocks);

    let (mut display, _backlight) = pins
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM4,
            &mut peripherals.MCLK,
            peripherals.TC2,
            &mut delay,
            &mut pins.port,
        )
        .unwrap();

    let mut adc1 = Adc::adc1(peripherals.ADC1, &mut peripherals.MCLK, &mut clocks, GCLK11);
    let mut joystick = pins.joystick.init(&mut pins.port);

    // list of "pixels" for mnist inference
    let mut pixels: [u32; 784] = [1; 784];

    // draw background
    (0..DISP_SIZE_X)
        .cartesian_product(0..DISP_SIZE_Y)
        .map(|point| Pixel(Point::new(point.0 as i32, point.1 as i32), RgbColor::BLACK))
        .draw(&mut display);

    // draw mnist canvas
    (0..28)
        .cartesian_product(0..28)
        .map(|point| mnist_pixel(point, &pixels))
        .draw(&mut display);

    // draw square at starting point
    let mut position = Point::new(13 * PIXEL_SIZE, 13 * PIXEL_SIZE);
    Rectangle::new(
        position,
        Point::new(position.x + PIXEL_SIZE, position.y + PIXEL_SIZE),
    )
    .into_styled(PrimitiveStyle::with_fill(CURSOR_COLOR))
    .draw(&mut display);

    loop {}
}

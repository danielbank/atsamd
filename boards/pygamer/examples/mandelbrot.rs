//! Generate a mandelbrot set and draw it to screen

#![no_std]
#![no_main]

#[allow(unused_imports)]
use panic_halt;
use pygamer as hal;

use embedded_graphics::drawable::Pixel;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use hal::adc::Adc;
use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::gclk::pchctrl::GEN_A::GCLK11;
use hal::pac::{CorePeripherals, Peripherals};
use hal::pins::Keys;
use itertools::Itertools;
use num::Complex;

use smart_leds::hsv::{hsv2rgb, Hsv};

/// The width and height of the display
const DISP_SIZE_X: i32 = 160;
const DISP_SIZE_Y: i32 = 128;

//todo good variable choices?
const CXMIN: f32 = -2f32;
const CXMAX: f32 = 1f32;
const CYMIN: f32 = -1.5f32;
const CYMAX: f32 = 1.5f32;
const SCALEX: f32 = (CXMAX - CXMIN) / DISP_SIZE_X as f32;
const SCALEY: f32 = (CYMAX - CYMIN) / DISP_SIZE_Y as f32;

fn to_pixels(triple: (i32, i32, u16)) -> Pixel<Rgb565> {
    let color = hsv2rgb(Hsv {
        hue: triple.2 as u8,
        sat: 255,
        val: 32,
    });

    Pixel(
        Point::new(triple.0 as i32, triple.1 as i32),
        Rgb565::new(color.r, color.g, color.b),
    )
}
fn mandelbrot(
    pair: (i32, i32),
    x_offset: f32,
    y_offset: f32,
    magnification: f32,
    iterations: u16,
) -> (i32, i32, u16) {
    let cx = CXMIN + x_offset + pair.0 as f32 * SCALEX * magnification;
    let cy = CYMIN + y_offset + pair.1 as f32 * SCALEY * magnification;

    let c = Complex::new(cx, cy);
    let mut z = Complex::new(0f32, 0f32);

    let mut i = 0;
    for t in 0..iterations {
        //todo manhattan norm ok?
        if z.l1_norm() > 2.0 {
            break;
        }
        z = z * z + c;
        i = t;
    }

    (pair.0, pair.1, i)
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
    let mut buttons = pins.buttons.init(&mut pins.port);

    let mut x_offset: f32 = 0.0;
    let mut y_offset: f32 = 0.0;
    let mut magnification: f32 = 1.0;
    let mut iterations: u16 = 256u16;

    // initial draw
    (0..DISP_SIZE_X)
        .cartesian_product(0..DISP_SIZE_Y)
        .map(|point| mandelbrot(point, x_offset, y_offset, magnification, iterations))
        .map(to_pixels)
        .draw(&mut display);

    loop {
        let (x, y) = joystick.read(&mut adc1);
        let x_delta: f32 = if x < 147 {
            -0.1
        } else if (x >= 147) && (x < 1048) {
            -0.01
        } else if (x >= 1048) && (x < 3048) {
            0.0
        } else if (x >= 3048) && (x < 3948) {
            0.01
        } else {
            0.1
        };
        x_offset += x_delta;

        let y_delta: f32 = if y < 147 {
            -0.1
        } else if (y >= 147) && (y < 1048) {
            -0.01
        } else if (y >= 1048) && (y < 3048) {
            0.0
        } else if (y >= 3048) && (y < 3948) {
            0.01
        } else {
            0.1
        };
        y_offset += y_delta;

        let magnification_old = magnification;
        let iterations_old = iterations;
        for event in buttons.events() {
            match event {
                Keys::BDown => {
                    magnification += 0.1;
                }
                Keys::ADown => {
                    if magnification > 0.1 {
                        magnification -= 0.1;
                    }
                }
                Keys::SelectDown => {
                    if iterations > 8 {
                        iterations -= 8;
                    }
                }
                Keys::StartDown => {
                    iterations += 8;
                }
                _ => {}
            }
        }

        if iterations != iterations_old
            || magnification != magnification_old
            || x_delta != 0.0
            || y_delta != 0.0
        {
            //draw background
            (0..DISP_SIZE_X)
                .cartesian_product(0..DISP_SIZE_Y)
                .map(|point| mandelbrot(point, x_offset, y_offset, magnification, iterations))
                .map(to_pixels)
                .draw(&mut display);
        }
    }
}

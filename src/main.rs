#![no_main]
#![no_std]

use cortex_m_rt::entry;
use defmt::println;
use microflow::model;
use nalgebra::matrix;

use {defmt_rtt as _, panic_probe as _};

#[model("sine.tflite")]
struct Sine;

#[entry]
fn main() -> ! {
    // Get device peripherals and the board abstraction.
    let dp = daisy::pac::Peripherals::take().unwrap();
    let board = daisy::Board::take().unwrap();

    // Configure board's peripherals.
    let ccdr = daisy::board_freeze_clocks!(board, dp);
    let pins = daisy::board_split_gpios!(board, ccdr, dp);
    let mut led_user = daisy::board_split_leds!(pins).USER;

    let one_second = ccdr.clocks.sys_ck().to_Hz();

    let mut x = 0.0;

    loop {
        if x > 2.0 {
            x = 0.0
        }
        x += 0.05;

        let y_predicted = Sine::predict(matrix![x])[0];

        println!("Predicted sin({}): {}", x, y_predicted);
        println!("Calculated sin({}): {}", x, libm::sinf(x));

        led_user.toggle();

        cortex_m::asm::delay(one_second);
    }
}

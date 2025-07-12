#![no_main]
#![no_std]

mod model;

use burn::{backend::NdArray, tensor::Tensor};
use cortex_m_rt::entry;

use crate::model::sine::Model;

use {defmt_rtt as _, panic_probe as _};

type Backend = NdArray<f32>;
type BackendDevice = <Backend as burn::tensor::backend::Backend>::Device;

use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
    // Setup Heap
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 100 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    // Get a default device for the backend
    let device = BackendDevice::default();

    // Create a new model and load the state
    let model: Model<Backend> = Model::default();

    // Get device peripherals and the board abstraction.
    let dp = daisy::pac::Peripherals::take().unwrap();
    let board = daisy::Board::take().unwrap();

    // Configure board's peripherals.
    let ccdr = daisy::board_freeze_clocks!(board, dp);
    let pins = daisy::board_split_gpios!(board, ccdr, dp);
    let mut led_user = daisy::board_split_leds!(pins).USER;

    defmt::println!("Done initializing model...");

    // Define input
    let mut input = 0.0;

    // Run inference in a loop
    loop {
        if input > 2.0 {
            input = 0.0
        }
        input += 0.05;

        // Run the model
        let output = run_model(&model, &device, input);

        // Output the values
        match output.into_data().as_slice::<f32>() {
            Ok(slice) => defmt::println!("input: {} - output: {}", input, slice),
            Err(err) => core::panic!("err: {:?}", err),
        };

        led_user.toggle();
    }
}

fn run_model<'a>(model: &Model<NdArray>, device: &BackendDevice, input: f32) -> Tensor<Backend, 2> {
    // Define the tensor
    let input = Tensor::<Backend, 2>::from_floats([[input]], &device);

    // Run the model on the input
    let output = model.forward(input);

    output
}

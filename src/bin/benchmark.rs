#![no_main]
#![no_std]

use core::hint::black_box;

use burn::{backend::NdArray, tensor::Tensor};
use daisy::hal as _;
use defmt_rtt as _;
use panic_probe as _;

pub mod sine {
    include!(concat!(env!("OUT_DIR"), "/model/sine.rs"));
}

use sine::Model;

type Backend = NdArray<f32>;
type BackendDevice = <Backend as burn::tensor::backend::Backend>::Device;

use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub const MS: u32 = 1_000;
pub const US: u32 = 1_000_000;
pub const NS: u32 = 1_000_000_000;

/// Terminates the application gracefully for probe-run debugger
/// Makes the debugger exit with success status (exit-code = 0)
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt(); // Trigger breakpoint instruction repeatedly
    }
}

#[macro_export]
macro_rules! bench_cycles {
    ( $cp:expr, $x:expr ) => {
        {
            use core::sync::atomic::{self, Ordering};
            use daisy::pac::DWT;

            $cp.DCB.enable_trace();
            $cp.DWT.enable_cycle_counter();

            atomic::compiler_fence(Ordering::Acquire);
            let before = DWT::cycle_count();
            $x
            let after = DWT::cycle_count();
            atomic::compiler_fence(Ordering::Release);

            if after >= before {
                after - before
            } else {
                after + (u32::MAX - before)
            }
        }
    };
}

#[macro_export]
macro_rules! bench_time {
    ( $cp:expr, $sysclk_hz:expr, $x:expr ) => {{
        let cycles = $crate::bench_cycles!($cp, $x);
        (cycles, (cycles as f32) / ($sysclk_hz as f32))
    }};
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Run Benchmark");

    // Setup Heap
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 100 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    // Get access to Cortex-M core peripherals (CPU-level hardware)
    let mut cortex_peripherals = cortex_m::Peripherals::take().unwrap();

    // Enable instruction and data caches for better performance
    cortex_peripherals.SCB.enable_icache();
    cortex_peripherals
        .SCB
        .enable_dcache(&mut cortex_peripherals.CPUID);

    // Get access to device-specific peripherals (board-level hardware)
    let device_peripherals = daisy::pac::Peripherals::take().unwrap();

    // Initialize the Daisy board
    let daisy_board = daisy::Board::take().unwrap();

    // Configure and freeze the clock settings for the board
    let clock_configuration = daisy::board_freeze_clocks!(daisy_board, device_peripherals);

    let pins = daisy::board_split_gpios!(daisy_board, clock_configuration, device_peripherals);
    let mut led_user = daisy::board_split_leds!(pins).USER;

    // Get the system clock frequency in Hz
    let system_clock_frequency_hz = clock_configuration.clocks.sys_ck().to_Hz();

    // Get a default device for the backend
    let device = BackendDevice::default();

    // Create a new model and load the state
    let model: Model<Backend> = Model::default();

    let mut x = 0.5;

    let one_second = system_clock_frequency_hz;
    // Loop infinite
    loop {
        if x > 2.0 {
            x = 0.0
        }
        x += 0.05;

        let (cycles, execution_time) =
            // Benchmark the dot product calculation and measure execution time
            bench_time!(cortex_peripherals, system_clock_frequency_hz, {
                // Run the model prediction
                let output = run_model(&model, &device, x);
                black_box(output);
            });

        defmt::println!("Cycles: {}", cycles);
        defmt::println!("Time: {} us", execution_time * US as f32);

        led_user.toggle();
        cortex_m::asm::delay(one_second);
    }
}

fn run_model<'a>(model: &Model<NdArray>, device: &BackendDevice, input: f32) -> f32 {
    // Define the tensor
    let input = Tensor::<Backend, 2>::from_floats([[input]], &device);

    // Run the model on the input
    let output = model.forward(input);

    output.into_data().as_slice().unwrap()[0]
}

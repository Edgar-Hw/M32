use std::{
    hint::black_box,
    process::ExitCode,
    time::{Duration, Instant},
};

const SCHEMA_VERSION: u32 = 1;
const WIDTH: usize = 240;
const HEIGHT: usize = 320;
const PIXELS: usize = WIDTH * HEIGHT;
const RGBA_BYTES: usize = PIXELS * 4;

const COPY_ITERATIONS: u64 = 4_000;
const RGB565_ITERATIONS: u64 = 1_000;
const SCALE_3X_ITERATIONS: u64 = 80;

fn main() -> ExitCode {
    println!("M32_BASELINE_BENCHMARK schema_version={SCHEMA_VERSION}");
    println!("logical_width={WIDTH}");
    println!("logical_height={HEIGHT}");

    let copy = benchmark_rgba_copy();
    let rgb565 = benchmark_rgb565_to_rgba();
    let scale = benchmark_integer_scale_3x();

    print_result(&copy);
    print_result(&rgb565);
    print_result(&scale);

    if [copy, rgb565, scale]
        .iter()
        .all(|result| result.elapsed > Duration::ZERO && result.checksum != 0)
    {
        println!("M32_BASELINE_BENCHMARK result=PASS");
        ExitCode::SUCCESS
    } else {
        eprintln!("M32_BASELINE_BENCHMARK result=FAIL");
        ExitCode::FAILURE
    }
}

#[derive(Debug, Clone)]
struct BenchResult {
    name: &'static str,
    iterations: u64,
    elapsed: Duration,
    checksum: u64,
}

fn print_result(result: &BenchResult) {
    let total_ns = result.elapsed.as_nanos();
    let ns_per_iteration = total_ns / u128::from(result.iterations);

    println!(
        "M32_BENCH name={} iterations={} total_ns={} ns_per_iteration={} checksum={}",
        result.name, result.iterations, total_ns, ns_per_iteration, result.checksum
    );
}

fn benchmark_rgba_copy() -> BenchResult {
    let mut source = vec![0_u8; RGBA_BYTES];
    let mut destination = vec![0_u8; RGBA_BYTES];

    for (index, byte) in source.iter_mut().enumerate() {
        *byte = ((index * 31 + 17) & 0xFF) as u8;
    }

    let start = Instant::now();
    let mut checksum = 0_u64;

    for iteration in 0..COPY_ITERATIONS {
        destination.copy_from_slice(black_box(&source));

        let probe_index = (iteration as usize * 977) % destination.len();
        checksum = checksum.wrapping_add(u64::from(black_box(destination[probe_index])));
    }

    BenchResult {
        name: "rgba_copy_240x320",
        iterations: COPY_ITERATIONS,
        elapsed: start.elapsed(),
        checksum: checksum.max(1),
    }
}

fn benchmark_rgb565_to_rgba() -> BenchResult {
    let source: Vec<u16> = (0..PIXELS)
        .map(|index| ((index * 73 + 0x1234) & 0xFFFF) as u16)
        .collect();
    let mut destination = vec![0_u8; RGBA_BYTES];

    let start = Instant::now();
    let mut checksum = 0_u64;

    for iteration in 0..RGB565_ITERATIONS {
        convert_rgb565_to_rgba(black_box(&source), black_box(&mut destination));

        let probe_index = ((iteration as usize * 613) % PIXELS) * 4;
        checksum = checksum
            .wrapping_mul(33)
            .wrapping_add(u64::from(destination[probe_index]))
            .wrapping_add(u64::from(destination[probe_index + 1]))
            .wrapping_add(u64::from(destination[probe_index + 2]));
    }

    BenchResult {
        name: "rgb565_to_rgba_240x320",
        iterations: RGB565_ITERATIONS,
        elapsed: start.elapsed(),
        checksum: checksum.max(1),
    }
}

fn convert_rgb565_to_rgba(source: &[u16], destination: &mut [u8]) {
    let (rgba_pixels, remainder) = destination.as_chunks_mut::<4>();
    debug_assert!(remainder.is_empty());

    for (pixel, rgba) in source.iter().zip(rgba_pixels.iter_mut()) {
        let red5 = ((pixel >> 11) & 0x1F) as u8;
        let green6 = ((pixel >> 5) & 0x3F) as u8;
        let blue5 = (pixel & 0x1F) as u8;

        rgba[0] = (red5 << 3) | (red5 >> 2);
        rgba[1] = (green6 << 2) | (green6 >> 4);
        rgba[2] = (blue5 << 3) | (blue5 >> 2);
        rgba[3] = 0xFF;
    }
}

fn benchmark_integer_scale_3x() -> BenchResult {
    let source: Vec<u32> = (0..PIXELS)
        .map(|index| {
            let value = (index as u32).wrapping_mul(2_654_435_761);
            value | 0xFF00_0000
        })
        .collect();

    let scaled_width = WIDTH * 3;
    let scaled_height = HEIGHT * 3;
    let mut destination = vec![0_u32; scaled_width * scaled_height];

    let start = Instant::now();
    let mut checksum = 0_u64;

    for iteration in 0..SCALE_3X_ITERATIONS {
        integer_scale_3x(black_box(&source), black_box(&mut destination));

        let probe_index = (iteration as usize * 7_919) % destination.len();
        checksum = checksum.wrapping_add(u64::from(destination[probe_index]));
    }

    BenchResult {
        name: "integer_scale_3x_240x320",
        iterations: SCALE_3X_ITERATIONS,
        elapsed: start.elapsed(),
        checksum: checksum.max(1),
    }
}

fn integer_scale_3x(source: &[u32], destination: &mut [u32]) {
    let scaled_width = WIDTH * 3;

    for source_y in 0..HEIGHT {
        let source_row = &source[source_y * WIDTH..(source_y + 1) * WIDTH];

        for repeat_y in 0..3 {
            let destination_y = source_y * 3 + repeat_y;
            let destination_row = &mut destination[destination_y * scaled_width..(destination_y + 1) * scaled_width];

            for (source_x, pixel) in source_row.iter().copied().enumerate() {
                let destination_x = source_x * 3;
                destination_row[destination_x..destination_x + 3].fill(pixel);
            }
        }
    }
}

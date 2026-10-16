#![no_std]
#![no_main]

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb888, RgbColor},
};
use spin::Mutex;
use uart_16550::{Config, Uart16550};

mod frame_buffer_embedded_graphics;
mod frame_buffer_info;
pub use frame_buffer_info::FrameBufferInfo;
mod limine_requests;
pub mod rgb_pixel_info;
pub use rgb_pixel_info::RgbPixelInfo;

use crate::{
    frame_buffer_embedded_graphics::FrameBufferEmbeddedGraphics,
    limine_requests::{FRAME_BUFFER_REQUEST, MP_REQUEST},
};

type SerialPort = Uart16550<uart_16550::backend::PioBackend>;

static UART: Mutex<Option<SerialPort>> = Mutex::new(None);

#[unsafe(no_mangle)]
unsafe extern "C" fn limine_mp_entrypoint(_cpu: &limine::mp::MpInfo) -> ! {
    uart_write(b"Hello from AP!\n\r");
    hlt_loop()
}

fn init_uart() {
    let mut uart = unsafe { Uart16550::new_port(0x3f8).unwrap() };
    uart.init(Config::default()).unwrap();
    *UART.lock() = Some(uart);
}

fn uart_write(bytes: &[u8]) {
    let mut guard = UART.lock();
    if let Some(uart) = guard.as_mut() {
        uart.send_bytes_exact(bytes);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(limine_requests::BASE_REVISION.is_supported());
    init_uart();
    uart_write(b"\n\nhello from BSP!\n\r");

    let mp_response = MP_REQUEST.response().unwrap();
    for cpu in mp_response.cpus() {
        cpu.bootstrap(limine_mp_entrypoint, 0);
    }

    let frame_buffer = FRAME_BUFFER_REQUEST.response().unwrap();
    if let Some(frame_buffer) = frame_buffer.framebuffers().iter().next() {
        let mut frame_buffer = {
            let addr = frame_buffer.address().addr().try_into().unwrap();
            let info = (*frame_buffer).into();
            unsafe { FrameBufferEmbeddedGraphics::new(addr, info) }
        };
        let bounding_box = embedded_graphics::geometry::Dimensions::bounding_box(&frame_buffer);
        frame_buffer
            .fill_solid(&bounding_box, Rgb888::YELLOW)
            .unwrap();
    }

    hlt_loop();
}

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hlt_loop();
}

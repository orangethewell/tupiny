#![no_std]
#![no_main]

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb888, RgbColor},
};
use uart_16550::{Config, Uart16550};

mod frame_buffer_embedded_graphics;
mod frame_buffer_info;
pub use frame_buffer_info::FrameBufferInfo;
mod limine_requests;
pub mod rgb_pixel_info;
pub use rgb_pixel_info::RgbPixelInfo;

use crate::{
    frame_buffer_embedded_graphics::FrameBufferEmbeddedGraphics,
    limine_requests::FRAME_BUFFER_REQUEST,
};

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(limine_requests::BASE_REVISION.is_supported());
    let mut uart = unsafe { Uart16550::new_port(0x3f8).unwrap() };
    uart.init(Config::default())
        .expect("should init device successfully");
    uart.send_bytes_exact(b"hello world!");
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

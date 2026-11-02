//! Pico 2 W: print "Hello from Pico 2 W" over USB serial once a second.
//!
//! No wiring needed — the USB cable is both power and the serial link.
//! On the Mac the board shows up as /dev/cu.usbmodem*.

#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Timer;
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use panic_halt as _;
use static_cell::StaticCell;

// Shown by `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 3] = [
    embassy_rp::binary_info::rp_program_name!(c"pico2_rust_demo"),
    embassy_rp::binary_info::rp_program_description!(c"Hello over USB serial"),
    embassy_rp::binary_info::rp_cargo_version!(),
];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type UsbDriver = Driver<'static, USB>;

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe); // test VID/PID
    config.manufacturer = Some("pico2_rust_demo");
    config.product = Some("Pico 2 W USB serial");
    config.serial_number = Some("0001");
    config.max_packet_size_0 = 64;

    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();

    let mut builder = embassy_usb::Builder::new(
        Driver::new(p.USB, Irqs),
        config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );
    let mut class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    spawner.spawn(usb_task(builder.build()).unwrap());

    let mut count: u32 = 0;
    loop {
        // Wait until a terminal on the Mac opens the port (raises DTR).
        // wait_connection() returns at once after enumeration, so poll with a
        // Timer: a loop with no real await would starve usb_task.
        class.wait_connection().await;
        if !class.dtr() {
            Timer::after_millis(50).await;
            continue;
        }
        while class.dtr() {
            let mut line = Line { buf: [0; 64], len: 0 };
            let _ = write!(line, "Hello from Pico 2 W #{count}\r\n");
            if class.write_packet(line.as_bytes()).await.is_err() {
                break;
            }
            count += 1;
            Timer::after_secs(1).await;
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, UsbDriver>) -> ! {
    usb.run().await
}

/// Fixed-size line buffer; one USB packet is at most 64 bytes.
struct Line {
    buf: [u8; 64],
    len: usize,
}

impl Line {
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl Write for Line {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len + s.len();
        let dst = self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        dst.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

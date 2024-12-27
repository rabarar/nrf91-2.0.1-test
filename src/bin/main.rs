#![no_std]
#![no_main]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::Duration;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{pac};
use {defmt_rtt as _, panic_probe as _};

use nrf_modem::{ConnectionPreference, LteLink, SystemMode, TcpStream};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use cortex_m::peripheral::NVIC;

#[allow(unused_imports)]
use tinyrlibc;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    defmt::info!("hello");
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_00, Level::Low, OutputDrive::Standard);
    led.set_high();

    // Set IPC RAM to nonsecure
    const SPU_REGION_SIZE: u32 = 0x2000; // 8kb
    const RAM_START: u32 = 0x2000_0000; // 256kb
    let spu = embassy_nrf::pac::SPU;
    let region_start = 0x2000_000 - RAM_START / SPU_REGION_SIZE;
    let region_end = region_start + (0x2000_8000 - 0x2000_0000) / SPU_REGION_SIZE;
    for i in region_start..region_end {
        spu.ramregion(i as usize).perm().write(|w| {
            w.set_execute(true);
            w.set_write(true);
            w.set_read(true);
            w.set_secattr(false);
            w.set_lock(false);
        })
    }

    // Set regulator access registers to nonsecure
    spu.periphid(4).perm().write(|w| w.set_secattr(false));
    // Set clock and power access registers to nonsecure
    spu.periphid(5).perm().write(|w| w.set_secattr(false));
    // Set IPC access register to nonsecure
    spu.periphid(42).perm().write(|w| w.set_secattr(false));


    use embassy_nrf::pac::interrupt;

    // Interrupt Handler for LTE related hardware. Defer straight to the library.
    #[interrupt]
    #[allow(non_snake_case)]
    fn IPC() {
        nrf_modem::ipc_irq_handler();
    }

    let mut cp = unwrap!(cortex_m::Peripherals::take());

    // Enable the modem interrupts
    unsafe {
        NVIC::unmask(pac::Interrupt::IPC);
        cp.NVIC.set_priority(pac::Interrupt::IPC, 0 << 5);
    }

    run().await;

    exit();
}


async fn run() {
    defmt::println!("Initializing modem");
    nrf_modem::init(SystemMode {
        lte_support: true,
        lte_psm_support: true,
        nbiot_support: true,
        gnss_support: true,
        preference: ConnectionPreference::None,
    })
    .await
    .unwrap();

    defmt::println!("Creating link");

    let link = LteLink::new().await.unwrap();
    embassy_time::with_timeout(Duration::from_millis(30000), link.wait_for_link())
        .await
        .unwrap()
        .unwrap();

    let google_ip = nrf_modem::get_host_by_name("google.com").await.unwrap();
    defmt::println!("Google ip: {:?}", defmt::Debug2Format(&google_ip));

    let stream = embassy_time::with_timeout(
        Duration::from_millis(2000),
        TcpStream::connect(SocketAddr::new(google_ip, 80)),
    )
    .await
    .unwrap()
    .unwrap();

    stream
        .write("GET / HTTP/1.0\nHost: google.com\r\n\r\n".as_bytes())
        .await
        .unwrap();
    let mut buffer = [0; 1024];
    let used = stream.receive(&mut buffer).await.unwrap();

    defmt::println!("Google page: {}", core::str::from_utf8(used).unwrap());

    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let socket = nrf_modem::UdpSocket::bind(SocketAddr::new(ip, 53))
        .await
        .unwrap();
    // Do a DNS request
    let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    socket
        .send_to(
            &[
                0xdb, 0x42, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77,
                0x77, 0x77, 0x0C, 0x6E, 0x6F, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74, 0x65, 0x72,
                0x6E, 0x03, 0x65, 0x64, 0x75, 0x00, 0x00, 0x01, 0x00, 0x01,
            ],

            SocketAddr::new(ip,53)
        )
        .await
        .unwrap();
    let (result, source) = socket.receive_from(&mut buffer).await.unwrap();

    defmt::println!("Result: {:X}", result);
    defmt::println!("Source: {}", defmt::Debug2Format(&source));
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}


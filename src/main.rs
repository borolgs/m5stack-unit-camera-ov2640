use esp_idf_svc::espnow::{EspNow, PeerInfo, BROADCAST};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, WifiDriver};
use std::sync::mpsc;
use std::time::Duration;

const FRAME_INTERVAL_MS: u64 = 100;
const NOW_CHANNEL: u8 = 11;

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Advertising,
    Streaming,
}

enum Event {
    Connect([u8; 6]),
    Disconnect,
}

#[allow(dead_code)]
pub mod protocol {
    pub const PREFIX: [u8; 2] = [0xCA, 0x3E];

    pub const MSG_CAMERA_READY: u8 = 0x01;
    pub const MSG_CONNECT: u8 = 0x02;
    pub const MSG_FRAME_CHUNK: u8 = 0x03;
    pub const MSG_DISCONNECT: u8 = 0x04;

    // header: magic(2) + msg_type(1) + frame_id(2) + chunk_idx(2) + total_chunks(2) = 9 bytes
    pub const CHUNK_HEADER_SIZE: usize = 9;
    pub const CHUNK_DATA_SIZE: usize = 250 - CHUNK_HEADER_SIZE; // 241 bytes

    #[derive(Debug)]
    pub enum Message<'a> {
        CameraReady,
        Connect,
        Disconnect,
        FrameChunk {
            frame_id: u16,
            chunk_idx: u16,
            total_chunks: u16,
            data: &'a [u8],
        },
    }

    pub fn decode(data: &[u8]) -> Option<Message<'_>> {
        if data.len() < 3 || data[0..2] != PREFIX {
            return None;
        }
        let msg_type = data[2];
        match msg_type {
            MSG_CAMERA_READY => Some(Message::CameraReady),
            MSG_CONNECT => Some(Message::Connect),
            MSG_DISCONNECT => Some(Message::Disconnect),
            MSG_FRAME_CHUNK if data.len() >= CHUNK_HEADER_SIZE => {
                let frame_id = u16::from_le_bytes([data[3], data[4]]);
                let chunk_idx = u16::from_le_bytes([data[5], data[6]]);
                let total_chunks = u16::from_le_bytes([data[7], data[8]]);
                let payload = &data[CHUNK_HEADER_SIZE..];
                Some(Message::FrameChunk {
                    frame_id,
                    chunk_idx,
                    total_chunks,
                    data: payload,
                })
            }
            _ => None,
        }
    }

    pub fn encode_camera_ready() -> [u8; 3] {
        [PREFIX[0], PREFIX[1], MSG_CAMERA_READY]
    }

    pub fn encode_connect() -> [u8; 3] {
        [PREFIX[0], PREFIX[1], MSG_CONNECT]
    }

    pub fn encode_disconnect() -> [u8; 3] {
        [PREFIX[0], PREFIX[1], MSG_DISCONNECT]
    }

    pub fn encode_frame_chunk(
        frame_id: u16,
        chunk_idx: u16,
        total_chunks: u16,
        data: &[u8],
    ) -> Vec<u8> {
        let mut packet = Vec::with_capacity(CHUNK_HEADER_SIZE + data.len());
        packet.extend_from_slice(&PREFIX);
        packet.push(MSG_FRAME_CHUNK);
        packet.extend_from_slice(&frame_id.to_le_bytes());
        packet.extend_from_slice(&chunk_idx.to_le_bytes());
        packet.extend_from_slice(&total_chunks.to_le_bytes());
        packet.extend_from_slice(data);
        packet
    }
}

fn main() -> anyhow::Result<()> {
    use protocol::*;

    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let mut wifi = WifiDriver::new(peripherals.modem, sysloop.clone(), None)?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ..Default::default()
    }))?;
    wifi.start()?;

    unsafe {
        esp_idf_svc::sys::esp_wifi_set_channel(
            NOW_CHANNEL,
            esp_idf_svc::sys::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE,
        );
    }

    let espnow = EspNow::take()?;
    espnow.add_peer(PeerInfo {
        peer_addr: BROADCAST,
        channel: NOW_CHANNEL,
        encrypt: false,
        ..Default::default()
    })?;

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let (send_done_tx, send_done_rx) = mpsc::sync_channel::<()>(1);

    espnow.register_recv_cb(move |info, data| match decode(data) {
        Some(Message::Connect) => {
            log::info!("Got connect from {:02X?}", info.src_addr);
            let _ = event_tx.send(Event::Connect(*info.src_addr));
        }
        Some(Message::Disconnect) => {
            log::info!("Got disconnect from {:02X?}", info.src_addr);
            let _ = event_tx.send(Event::Disconnect);
        }
        _ => {}
    })?;

    espnow.register_send_cb(move |_mac, _status| {
        let _ = send_done_tx.try_send(());
    })?;

    // https://docs.m5stack.com/en/unit/Unit%20Camera#pinmap
    let camera_config = esp_idf_sys::camera::camera_config_t {
        pin_pwdn: -1,
        pin_reset: 15,
        pin_xclk: 27,
        sccb_i2c_port: -1,
        __bindgen_anon_1: esp_idf_sys::camera::camera_config_t__bindgen_ty_1 { pin_sccb_sda: 25 },
        __bindgen_anon_2: esp_idf_sys::camera::camera_config_t__bindgen_ty_2 { pin_sscb_scl: 23 },
        pin_d7: 19,
        pin_d6: 36,
        pin_d5: 18,
        pin_d4: 39,
        pin_d3: 5,
        pin_d2: 34,
        pin_d1: 35,
        pin_d0: 32,
        pin_vsync: 22,
        pin_href: 26,
        pin_pclk: 21,
        xclk_freq_hz: 10_000_000,
        ledc_timer: esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
        ledc_channel: esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,
        pixel_format: esp_idf_sys::camera::pixformat_t_PIXFORMAT_JPEG,
        frame_size: esp_idf_sys::camera::framesize_t_FRAMESIZE_QQVGA,
        jpeg_quality: 15,
        fb_count: 2,
        fb_location: esp_idf_sys::camera::camera_fb_location_t_CAMERA_FB_IN_DRAM,
        grab_mode: esp_idf_sys::camera::camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,
    };

    unsafe {
        if esp_idf_sys::camera::esp_camera_init(&camera_config) != 0 {
            anyhow::bail!("camera init failed");
        }
    }

    let mut state = State::Advertising;
    let mut client_mac: Option<[u8; 6]> = None;
    let mut frame_id: u16 = 0;

    log::info!("Camera initialized, entering main loop");

    loop {
        match state {
            State::Advertising => {
                log::info!("Advertising...");
                espnow.send(BROADCAST, &encode_camera_ready())?;

                match event_rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(Event::Connect(mac)) => {
                        espnow.add_peer(PeerInfo {
                            peer_addr: mac,
                            channel: NOW_CHANNEL,
                            encrypt: false,
                            ..Default::default()
                        })?;
                        client_mac = Some(mac);
                        frame_id = 0;
                        state = State::Streaming;
                        log::info!("Client connected: {:02X?}", mac);
                    }
                    _ => {}
                }
            }

            State::Streaming => {
                if let Ok(Event::Disconnect) = event_rx.try_recv() {
                    if let Some(mac) = client_mac.take() {
                        let _ = espnow.del_peer(mac);
                    }
                    state = State::Advertising;
                    log::info!("Client disconnected, returning to advertising");
                    continue;
                }

                let mac = client_mac.unwrap();

                let fb = unsafe { esp_idf_sys::camera::esp_camera_fb_get() };
                if fb.is_null() {
                    log::warn!("Failed to get frame");
                    continue;
                }

                let data = unsafe { std::slice::from_raw_parts((*fb).buf, (*fb).len) };
                let total_chunks = (data.len() + CHUNK_DATA_SIZE - 1) / CHUNK_DATA_SIZE;

                for (chunk_idx, chunk) in data.chunks(CHUNK_DATA_SIZE).enumerate() {
                    let packet =
                        encode_frame_chunk(frame_id, chunk_idx as u16, total_chunks as u16, chunk);
                    if let Err(e) = espnow.send(mac, &packet) {
                        log::warn!("Send failed: {:?}", e);
                    }
                    let _ = send_done_rx.recv();
                }

                unsafe { esp_idf_sys::camera::esp_camera_fb_return(fb) };

                frame_id = frame_id.wrapping_add(1);
                std::thread::sleep(Duration::from_millis(FRAME_INTERVAL_MS));
            }
        }
    }
}

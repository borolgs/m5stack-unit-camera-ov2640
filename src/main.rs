fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let camera_config = esp_idf_sys::camera::camera_config_t {
        pin_pwdn: 32,
        pin_reset: -1,
        pin_xclk: 0,
        sccb_i2c_port: -1,
        __bindgen_anon_1: esp_idf_sys::camera::camera_config_t__bindgen_ty_1 { pin_sccb_sda: 26 },
        __bindgen_anon_2: esp_idf_sys::camera::camera_config_t__bindgen_ty_2 { pin_sscb_scl: 27 },
        pin_d7: 35,
        pin_d6: 34,
        pin_d5: 39,
        pin_d4: 36,
        pin_d3: 21,
        pin_d2: 19,
        pin_d1: 18,
        pin_d0: 5,
        pin_vsync: 25,
        pin_href: 23,
        pin_pclk: 22,
        xclk_freq_hz: 20000000,
        ledc_timer: esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
        ledc_channel: esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,
        pixel_format: esp_idf_sys::camera::pixformat_t_PIXFORMAT_JPEG,
        frame_size: esp_idf_sys::camera::framesize_t_FRAMESIZE_QVGA,
        jpeg_quality: 12,
        fb_count: 1,
        fb_location: esp_idf_sys::camera::camera_fb_location_t_CAMERA_FB_IN_PSRAM,
        grab_mode: esp_idf_sys::camera::camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,
    };

    unsafe {
        if esp_idf_sys::camera::esp_camera_init(&camera_config) != 0 {
            log::error!("camera init failed!");
            return;
        } else {
            log::info!("camera ready!");
        }

        let fb = esp_idf_sys::camera::esp_camera_fb_get();
        log::info!("Picture taken! Its size was: {} bytes", (*fb).len);
        let data = std::slice::from_raw_parts((*fb).buf, (*fb).len);
        log::info!("{data:?}");
    }
}

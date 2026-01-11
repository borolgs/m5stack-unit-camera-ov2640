fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // M5Stack Unit Camera (OV2640) pin configuration
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
        xclk_freq_hz: 20000000,
        ledc_timer: esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
        ledc_channel: esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,
        pixel_format: esp_idf_sys::camera::pixformat_t_PIXFORMAT_JPEG,
        frame_size: esp_idf_sys::camera::framesize_t_FRAMESIZE_QQVGA,
        jpeg_quality: 12,
        fb_count: 1,
        fb_location: esp_idf_sys::camera::camera_fb_location_t_CAMERA_FB_IN_DRAM,
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
        // log::info!("{data:?}");
    }
}

# M5Stack Camera Unit (ESP-WROOM-32E, OV2640)

Firmware for [M5Stack Unit Camera](https://docs.m5stack.com/en/unit/Unit%20Camera), based on esp32-camera bindings.

This project is specific: made exclusively for streaming low-resolution images via ESP-NOW.

If you just need a camera setup example, here's all you need (or check the references below):

```rs
let camera_config = esp_idf_sys::camera::camera_config_t { .. };

unsafe {
    if esp_idf_sys::camera::esp_camera_init(&camera_config) != 0 {
        anyhow::bail!("camera init failed");
    }

    let fb = esp_idf_sys::camera::esp_camera_fb_get();
    let data = std::slice::from_raw_parts((*fb).buf, (*fb).len);
    
    esp_idf_sys::camera::esp_camera_fb_return(fb);
}
```

References:
- https://www.reddit.com/r/esp32/comments/w8kn8z/comment/kh7vm1y
- https://github.com/jlocash/esp-camera-rs

// Crutch! Only works for Vulkan.
// -----------------------------
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

static VULKAN_CUSTOM_BACKGROUND_COLOR: OnceLock<AtomicU32> = OnceLock::new();

/// Crutch!
/// Sets the background color.
/// Only has an effect if TextureColor::OpaqueBlack
/// is selected (Vulkan only).
pub fn set_background_color(r: u8, g: u8, b: u8, a: u8) {
    let color = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24);
    VULKAN_CUSTOM_BACKGROUND_COLOR
    .get_or_init(|| AtomicU32::new(color))
    .store(color, Ordering::Relaxed);
}

pub(crate) fn get_background_color() -> [f32; 4] {
    let color = VULKAN_CUSTOM_BACKGROUND_COLOR.get_or_init(|| AtomicU32::new(0xFF000000)).load(Ordering::Relaxed);
    [
        ((color >> 0) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 24) & 0xFF) as f32 / 255.0,
    ]
}
// -----------------------------
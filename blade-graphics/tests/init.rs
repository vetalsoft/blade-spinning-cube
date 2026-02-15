#[cfg(test)]
mod tests {
    use blade_graphics as gpu; // Предположим, корень библиотеки доступен так

    #[test]
    fn test_context_initialization() {
        // 1. Описываем, что нам нужно от графики
        let desc = gpu::ContextDesc {
            validation: true, // Проверим те самые VK_LAYER_KHRONOS_validation
            capture: false,
            ..Default::default()
        };

        // 2. Пробуем создать контекст (инстанс Vulkan)
        // Если тут будет ошибка в путях или версиях — тест упадет.
        let context = unsafe { gpu::Context::init(desc) }.expect("Failed to init Vulkan");
    
        // В blade-graphics инфо о железе обычно здесь:
        let info = context.device_information();
        println!("Используемая видеокарта: {}", info.device_name);

        // А возможности (capabilities) проверяем отдельно, если нужно
        let caps = context.capabilities();
        println!("Поддержка Ray Query: {:?}", caps.ray_query);

        assert!(!info.device_name.is_empty());
    }
}
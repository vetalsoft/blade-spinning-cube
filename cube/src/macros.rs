/// Макрос для упрощения работы с буфером глубины и дублирование кода в функциях:
///   [`CubeApp::new`] и [`CubeApp::resize`]
///
/// # Варианты использования:
///
/// * `depth!(desc size)` — генерирует [`gpu::TextureDesc`] на основе переданного размера.
/// * `depth!(view_desc)` — генерирует [`gpu::TextureViewDesc`] со стандартными параметрами.
/// * `depth!(create context, size)` — выполняет полный цикл инициализации: создает
///   текстуру и вьюху, возвращает кортеж `(Texture, TextureView)`.
///
/// # Примеры:
/// ```
/// // Полная инициализация одной строкой.
/// let (texture, view) = depth!(create context, window_size);
/// 
/// // Или ручное управление при изменении размера окна.
/// self.depth_texture = context.create_texture(depth!(desc new_size));
/// ```
macro_rules! depth {
    // Создаём TextureDesc (передаем только размер)
    (desc $size:expr) => {
        gpu::TextureDesc {
            name: "depth_texture",
            format: Depth32Float,
            size: gpu::Extent {
                width: $size.width,
                height: $size.height,
                depth: 1,
            },
            dimension: gpu::TextureDimension::D2,
            array_layer_count: 1,
            mip_level_count: 1,
            usage: gpu::TextureUsage::TARGET,
            sample_count: 1,
            external: None,
        }
    };

    // Создаём TextureViewDesc (без аргументов)
    (view_desc) => {
        gpu::TextureViewDesc {
            name: "depth_view",
            format: gpu::TextureFormat::Depth32Float,
            dimension: gpu::ViewDimension::D2,
            subresources: &gpu::TextureSubresources::default(),
        }
    };

    // Создаём всё (контекст + размер) -> возвращает (Texture, View)
    (create $context:expr, $size:expr) => {{
        let texture = $context.create_texture(depth!(desc $size));
        let view = $context.create_texture_view(texture, depth!(view_desc));
        (texture, view)
    }};
}
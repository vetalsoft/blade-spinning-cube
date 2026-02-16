#![windows_subsystem = "windows"]
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use blade_graphics::{self as gpu, Vertex, TextureFormat::Depth32Float};
use blade_util::create_static_buffer;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::time::Instant;

use blade_graphics::background_color_vulkan::set_background_color;

#[macro_use]
mod macros;

// Вершинные данные с нормалями
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct VertexData {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[VertexData] = &[
    // Front face (+Z) - красный
    VertexData { pos: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0], color: [1.0, 0.0, 0.0] },
    VertexData { pos: [ 0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0], color: [1.0, 0.0, 0.0] },
    VertexData { pos: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0], color: [1.0, 0.0, 0.0] },
    VertexData { pos: [-0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0], color: [1.0, 0.0, 0.0] },
    // Back face (-Z) - зеленый
    VertexData { pos: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0], color: [0.0, 1.0, 0.0] },
    VertexData { pos: [-0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0], color: [0.0, 1.0, 0.0] },
    VertexData { pos: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0], color: [0.0, 1.0, 0.0] },
    VertexData { pos: [ 0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0], color: [0.0, 1.0, 0.0] },
    // Left face (-X) - синий
    VertexData { pos: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
    VertexData { pos: [-0.5, -0.5,  0.5], normal: [-1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
    VertexData { pos: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
    VertexData { pos: [-0.5,  0.5, -0.5], normal: [-1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
    // Right face (+X) - желтый
    VertexData { pos: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 0.0] },
    VertexData { pos: [ 0.5, -0.5, -0.5], normal: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 0.0] },
    VertexData { pos: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 0.0] },
    VertexData { pos: [ 0.5,  0.5,  0.5], normal: [ 1.0,  0.0,  0.0], color: [1.0, 1.0, 0.0] },
    // Top face (+Y) - голубой
    VertexData { pos: [-0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 1.0] },
    VertexData { pos: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 1.0] },
    VertexData { pos: [ 0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 1.0] },
    VertexData { pos: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 1.0] },
    // Bottom face (-Y) - пурпурный
    VertexData { pos: [-0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0], color: [1.0, 0.0, 1.0] },
    VertexData { pos: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0], color: [1.0, 0.0, 1.0] },
    VertexData { pos: [ 0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0], color: [1.0, 0.0, 1.0] },
    VertexData { pos: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0], color: [1.0, 0.0, 1.0] },
];

const INDICES: &[u16] = &[
    // Front face
    0, 1, 2, 2, 3, 0,
    // Back face
    4, 5, 6, 6, 7, 4,
    // Left face
    8, 9, 10, 10, 11, 8,
    // Right face
    12, 13, 14, 14, 15, 12,
    // Top face
    16, 17, 18, 18, 19, 16,
    // Bottom face
    20, 21, 22, 22, 23, 20,
];

// Uniform-данные с параметрами света
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Globals {
    mvp_matrix: [[f32; 4]; 4],
    model_matrix: [[f32; 4]; 4],
    light_pos: [f32; 4],
    light_color: [f32; 4],
    ambient: [f32; 4],
    specular_power: f32,
    specular_intensity: f32,
    _pad: [f32; 2],
}

#[derive(blade_macros::ShaderData)]
struct CubeUniforms {
    globals: Globals,
}

#[derive(blade_macros::Vertex)]
struct CubeVertex {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

// Состояние приложения
struct AppState {
    light_pos: Vec3,
    camera_pos: Vec3,
    cube_rotation: f32,
}

// Основная структура приложения
struct CubeApp {
    context: gpu::Context,
    surface: gpu::Surface,
    pipeline: gpu::RenderPipeline,
    command_encoder: gpu::CommandEncoder,
    vertex_buf: gpu::Buffer,
    index_buf: gpu::Buffer,
    depth_texture: gpu::Texture,
    depth_view: gpu::TextureView,
    window_size: winit::dpi::PhysicalSize<u32>,
    prev_sync_point: Option<gpu::SyncPoint>,
    start_time: Instant,
    state: AppState,
}

impl CubeApp {
    fn make_surface_config(size: winit::dpi::PhysicalSize<u32>) -> gpu::SurfaceConfig {
        gpu::SurfaceConfig {
            size: gpu::Extent {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            usage: gpu::TextureUsage::TARGET,
            display_sync: gpu::DisplaySync::Recent,
            transparent: false,
            allow_exclusive_full_screen: false,
            color_space: gpu::ColorSpace::Srgb,
        }
    }

    fn new(window: &Window) -> Self {
        let context = gpu::Context::init(gpu::ContextDesc {
            validation: cfg!(debug_assertions),
            presentation: true,
            overlay: false,
            capture: false,
            timing: false,
            device_id: 0,
        }).unwrap();

        let window_size = window.inner_size();
        let surface = context
            .create_surface_configured(window, Self::make_surface_config(window_size))
            .unwrap();

        // Шейдер
        // let shader_source = std::fs::read_to_string("cube/src/cube.wgsl").unwrap();
        let shader_source = include_str!("cube.wgsl").to_string();
        let shader = context.create_shader(gpu::ShaderDesc {
            source: &shader_source,
        });

        let uniform_layout = <CubeUniforms as gpu::ShaderData>::layout();

        let vertex_buf = create_static_buffer(&context, "cube_vertex", VERTICES);
        context.sync_buffer(vertex_buf);

        let index_buf = create_static_buffer(&context, "cube_index", INDICES);
        context.sync_buffer(index_buf);

        let (depth_texture, depth_view) = depth!(create context, window_size);

        // Пайплайн
        let pipeline = context.create_render_pipeline(gpu::RenderPipelineDesc {
            name: "cube",
            data_layouts: &[&uniform_layout],
            vertex: shader.at("vs_main"),
            vertex_fetches: &[gpu::VertexFetchState {
                layout: &CubeVertex::layout(),
                instanced: false,
            }],
            primitive: gpu::PrimitiveState {
                topology: gpu::PrimitiveTopology::TriangleList,
                front_face: gpu::FrontFace::Ccw,
                cull_mode: Some(gpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(gpu::DepthStencilState {
                format: gpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: gpu::CompareFunction::Less,
                stencil: gpu::StencilState::default(),
                bias: gpu::DepthBiasState::default(),
            }),
            fragment: Some(shader.at("fs_main")),
            color_targets: &[gpu::ColorTargetState {
                format: surface.info().format,
                blend: None,
                write_mask: gpu::ColorWrites::ALL,
            }],
            multisample_state: gpu::MultisampleState::default(),
        });

        let command_encoder = context.create_command_encoder(gpu::CommandEncoderDesc {
            name: "main",
            buffer_count: 2,
        });

        // Начальное состояние
        let state = AppState {
            light_pos: Vec3::new(2.0, 3.0, 2.0),
            camera_pos: Vec3::new(1.9, 2.0, 1.9),
            cube_rotation: 0.0,
        };

        println!("=== Управление источником света ===");
        println!("WASD: движение по X/Z (плоскость)");
        println!("Q/E: вверх/вниз");
        println!("R: сброс позиции");
        println!("Начальная позиция света: {:?}", state.light_pos);

        // Мой костыль. Цвет фона
        set_background_color(50, 50, 50, 255);

        Self {
            context,
            surface,
            pipeline,
            command_encoder,
            vertex_buf,
            index_buf,
            depth_texture,
            depth_view,
            window_size,
            prev_sync_point: None,
            start_time: Instant::now(),
            state,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        let speed = 0.5;
        // let old_pos = self.state.light_pos;

        match key {
            KeyCode::KeyW => self.state.light_pos.z -= speed,
            KeyCode::KeyS => self.state.light_pos.z += speed,
            KeyCode::KeyA => self.state.light_pos.x -= speed,
            KeyCode::KeyD => self.state.light_pos.x += speed,
            KeyCode::KeyQ => self.state.light_pos.y += speed,
            KeyCode::KeyE => self.state.light_pos.y -= speed,
            KeyCode::KeyR => {
                self.state.light_pos = Vec3::new(2.0, 3.0, 2.0);
                println!("Сброс позиции света");
            }
            _ => return,
        }
        
        println!("Позиция света: ({:.1}, {:.1}, {:.1})", 
            self.state.light_pos.x, 
            self.state.light_pos.y, 
            self.state.light_pos.z);
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.window_size = size;

        // Пересоздаём текстуру глубины
        self.context.destroy_texture_view(self.depth_view);
        self.context.destroy_texture(self.depth_texture);

        (self.depth_texture, self.depth_view) = depth!(create self.context, size);

        let config = Self::make_surface_config(size);
        self.context.reconfigure_surface(&mut self.surface, config);
    }

    fn render(&mut self) {
        if self.window_size.width == 0 || self.window_size.height == 0 {
            return;
        }

        // Время для анимации куба
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.state.cube_rotation = elapsed * 0.5;
        
        // Матрицы трансформации
        let aspect = self.window_size.width as f32 / self.window_size.height as f32;
        let projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        let view = Mat4::look_at_rh(
            self.state.camera_pos,
            Vec3::ZERO,
            Vec3::Y,
        );
        let model = Mat4::from_rotation_y(self.state.cube_rotation);
        let mvp = projection * view * model;

        // Параметры освещения (позицию можжно изменять с клавиатуры)
        let uniforms = CubeUniforms {
            globals: Globals {
                mvp_matrix: mvp.to_cols_array_2d(),
                model_matrix: model.to_cols_array_2d(),
                light_pos: [self.state.light_pos.x, self.state.light_pos.y, self.state.light_pos.z, 1.0],
                light_color: [1.5, 1.5, 1.5, 1.0],
                ambient: [0.2, 0.2, 0.2, 1.0],
                specular_power: 8.0,
                specular_intensity: 0.2,
                _pad: [0.0; 2],
            },
        };

        // Рендер
        let frame = self.surface.acquire_frame();

        self.command_encoder.start();
        self.command_encoder.init_texture(frame.texture());
        self.command_encoder.init_texture(self.depth_texture);

        let mut pass = self.command_encoder.render(
            "cube",
            gpu::RenderTargetSet {
                colors: &[gpu::RenderTarget {
                    view: frame.texture_view(),
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::OpaqueBlack),
                    finish_op: gpu::FinishOp::Store,
                }],
                depth_stencil: Some(gpu::RenderTarget {
                    view: self.depth_view,
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::White),
                    finish_op: gpu::FinishOp::Store,
                }),
            },
        );

        let mut rc = pass.with(&self.pipeline);
        
        rc.bind(0, &uniforms);
        rc.bind_vertex(0, self.vertex_buf.at(0));
        rc.draw_indexed(
            self.index_buf.at(0),
            gpu::IndexType::U16,
            INDICES.len() as u32,
            0,
            0,
            1,
        );
        
        drop(rc);
        drop(pass);

        self.command_encoder.present(frame);
        let sync_point = self.context.submit(&mut self.command_encoder);
        
        if let Some(sp) = self.prev_sync_point.take() {
            self.context.wait_for(&sp, !0);
        }
        self.prev_sync_point = Some(sync_point);
    }

    fn deinit(&mut self) {
        if let Some(sp) = self.prev_sync_point.take() {
            self.context.wait_for(&sp, !0);
        }

        self.context.destroy_buffer(self.vertex_buf);
        self.context.destroy_buffer(self.index_buf);
        self.context.destroy_texture_view(self.depth_view);
        self.context.destroy_texture(self.depth_texture);
        self.context.destroy_command_encoder(&mut self.command_encoder);
        self.context.destroy_render_pipeline(&mut self.pipeline);
        self.context.destroy_surface(&mut self.surface);
    }
}

// App структура для winit
struct App {
    window: Option<Window>,
    cube: Option<CubeApp>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            cube: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("Blade Cube - Movable Ligh (WASD+QE, R)")
            .with_inner_size(winit::dpi::LogicalSize::new(500.0, 500.0));
        
        let window = event_loop
            .create_window(window_attrs)
            .expect("Failed to create window");

        let cube = CubeApp::new(&window);

        self.window = Some(window);
        self.cube = Some(cube);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(mut cube) = self.cube.take() {
            cube.deinit();
        }
        self.window = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(mut cube) = self.cube.take() {
            cube.deinit();
        }
        self.window = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let Some(cube) = self.cube.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event,
                ..
            } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        cube.handle_key(key_code);
                    }
                }
            }
            WindowEvent::Resized(size) => {
                cube.resize(size);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                cube.render();
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Failed to run app");
}
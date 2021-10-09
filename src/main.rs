use app::{App, AppStage, Assets, EventReader, Events, ParRunnable, SystemBuilder};
use render_plugin::{Pipeline, RenderPlugin, Shader};
use window_plugin::{
    winit::event::{ElementState, VirtualKeyCode},
    WindowCreateRequest, WindowDescriptor, WindowKeyboardInput, WindowPlugin,
};

fn create_window() -> impl ParRunnable {
    let mut event_reader = EventReader::<WindowKeyboardInput>::default();

    SystemBuilder::new()
        .on_stage(AppStage::PreUpdate)
        .read_resource::<Events<WindowKeyboardInput>>()
        .write_resource::<Events<WindowCreateRequest>>()
        .build(
            move |_, _, (keyboard_input_events, window_create_events), _| {
                for event in event_reader.iter(keyboard_input_events) {
                    if event.key_code == Some(VirtualKeyCode::A)
                        && event.state == ElementState::Released
                    {
                        window_create_events.send(WindowCreateRequest {
                            descriptor: WindowDescriptor {
                                width: 550,
                                height: 400,
                                title: "Another window".to_string(),
                            },
                        })
                    }
                }
            },
        )
}

fn startup_shader() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(AppStage::Startup)
        .write_resource::<Assets<Shader>>()
        .write_resource::<Assets<Pipeline>>()
        .build(|world, cmd, (shader_assets, material_assets), _| {
            let shader = render_plugin::Shader {
                source: String::from(include_str!("shader.wgsl")),
                stage: render_plugin::ShaderStages::VERTEX | render_plugin::ShaderStages::FRAGMENT,
            };

            let shader_handle = shader_assets.add(shader);

            let material = render_plugin::Pipeline {
                buffers: vec![render_plugin::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                    attributes: vec![render_plugin::VertexAttribute {
                        format: render_plugin::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
                vertex: render_plugin::VertexState {
                    shader: shader_handle.clone(),
                    entry_point: "main".to_string(),
                },
                fragment: Some(render_plugin::FragmentState {
                    shader: shader_handle.clone(),
                    entry_point: "main".to_string(),
                }),
                multisample: render_plugin::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            };
            let mat_handle = material_assets.add(material);

            cmd.spawn(world).add(mat_handle).entity();
        })
}

fn main() {
    App::new()
        .add_plugin(WindowPlugin::with_initial(WindowDescriptor {
            width: 800,
            height: 600,
            title: "WiGame".to_string(),
        }))
        .add_plugin(RenderPlugin::default())
        .add_system(startup_shader())
        .add_system(create_window())
        .run();
}

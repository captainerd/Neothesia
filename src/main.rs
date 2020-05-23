mod wgpu_jumpstart;
use wgpu_jumpstart::{Gpu, Uniform, Window};

mod ui;
use ui::Ui;

mod scene;
use scene::{InputEvent, Scene, SceneEvent, SceneType};

mod time_menager;
use time_menager::TimeMenager;

mod midi_device;

mod transform_uniform;
use transform_uniform::TransformUniform;

use wgpu_glyph::Section;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use std::rc::Rc;

mod rectangle_pipeline;

pub struct MainState {
    pub window_size: (f32, f32),
    pub mouse_pos: (f32, f32),
    /// Mouse Was Clicked This Frame
    pub mouse_clicked: bool,
    /// Mouse Is Pressed This Frame
    pub mouse_pressed: bool,
    pub time_menager: TimeMenager,
    pub transform_uniform: Uniform<TransformUniform>,

    pub midi_file: Option<Rc<lib_midi::Midi>>,
}

impl MainState {
    fn new(gpu: &Gpu) -> Self {
        Self {
            window_size: (0.0, 0.0),
            mouse_pos: (0.0, 0.0),
            mouse_clicked: false,
            mouse_pressed: false,
            time_menager: TimeMenager::new(),
            transform_uniform: Uniform::new(
                &gpu.device,
                TransformUniform::default(),
                wgpu::ShaderStage::VERTEX,
            ),
            midi_file: None,
        }
    }
    fn resize(&mut self, gpu: &mut Gpu, w: f32, h: f32) {
        self.window_size = (w, h);
        self.transform_uniform.data.update(w, h);
        self.transform_uniform.update(&mut gpu.encoder, &gpu.device);
    }
    fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_pos = (x, y);
    }
    fn update_mouse_pressed(&mut self, state: bool) {
        self.mouse_pressed = state;

        if state {
            self.update_mouse_clicked(true);
        }
    }
    fn update_mouse_clicked(&mut self, clicked: bool) {
        self.mouse_clicked = clicked;
    }
}

enum AppEvent<'a> {
    WindowEvent(&'a WindowEvent<'a>, &'a mut ControlFlow),
    SceneEvent(SceneEvent),
}

struct App<'a> {
    pub window: Window,
    pub gpu: Gpu,
    pub ui: Ui<'a>,
    pub main_state: MainState,
    game_scene: Box<scene::scene_transition::SceneTransition>,
}

impl<'a> App<'a> {
    fn new(mut gpu: Gpu, window: Window) -> Self {
        let main_state = MainState::new(&gpu);

        let ui = Ui::new(&main_state, &mut gpu);
        let game_scene = scene::menu_scene::MenuScene::new(&mut gpu);
        let game_scene = Box::new(scene::scene_transition::SceneTransition::new(Box::new(
            game_scene,
        )));

        Self {
            window,
            gpu,
            ui,
            main_state,
            game_scene,
        }
    }
    fn event(&mut self, event: AppEvent) {
        match event {
            AppEvent::WindowEvent(event, control_flow) => {
                match event {
                    WindowEvent::Resized(_) => {
                        self.resize();
                        self.gpu.submit();
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        self.window.on_dpi(*scale_factor);
                        // TODO: Check if this update is needed;
                        self.resize();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let dpi = &self.window.dpi;
                        let x = (position.x / dpi).round();
                        let y = (position.y / dpi).round();

                        self.main_state.update_mouse_pos(x as f32, y as f32);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if let winit::event::ElementState::Pressed = state {
                            self.main_state.update_mouse_pressed(true);
                        } else {
                            self.main_state.update_mouse_pressed(false);
                        }
                        self.game_scene.mouse_input(state, button);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == winit::event::ElementState::Released {
                            match input.virtual_keycode {
                                Some(winit::event::VirtualKeyCode::Escape) => {
                                    self.go_back(control_flow);
                                }
                                Some(key) => {
                                    let ae = AppEvent::SceneEvent(self.game_scene.input_event(
                                        &mut self.main_state,
                                        InputEvent::KeyReleased(key),
                                    ));
                                    self.event(ae);
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            AppEvent::SceneEvent(event) => match event {
                SceneEvent::MainMenu(event) => match event {
                    scene::menu_scene::Event::MidiOpen(port) => {
                        let state = scene::playing_scene::PlayingScene::new(
                            &mut self.gpu,
                            &mut self.main_state,
                            port,
                        );
                        self.game_scene.transition_to(Box::new(state));

                        // Self::render() is called right after this so we need to update scene here to prevent transition from not being visible for the first frame
                        self.game_scene
                            .update(&mut self.main_state, &mut self.gpu, &mut self.ui);
                    }
                },
                _ => {}
            },
        }
    }
    fn resize(&mut self) {
        self.window.on_resize(&mut self.gpu);
        let (w, h) = self.window.size();

        self.main_state.resize(&mut self.gpu, w, h);
        self.game_scene.resize(&mut self.main_state, &mut self.gpu);
        self.ui.resize(&self.main_state, &mut self.gpu);
    }
    fn go_back(&mut self, control_flow: &mut ControlFlow) {
        match self.game_scene.scene_type() {
            SceneType::MainMenu => {
                *control_flow = ControlFlow::Exit;
            }
            SceneType::Playing => {
                let state = scene::menu_scene::MenuScene::new(&mut self.gpu);
                self.game_scene.transition_to(Box::new(state));
            }
            SceneType::Transition => {}
        }
    }
    fn update(&mut self) {
        self.main_state.time_menager.update();

        let event = self
            .game_scene
            .update(&mut self.main_state, &mut self.gpu, &mut self.ui);

        self.event(AppEvent::SceneEvent(event));

        self.queue_fps();
    }
    fn render(&mut self) {
        let frame = self.window.surface.get_next_texture();

        self.clear(&frame);

        self.game_scene
            .render(&mut self.main_state, &mut self.gpu, &frame);

        self.ui.render(&mut self.main_state, &mut self.gpu, &frame);

        self.gpu.submit();

        self.main_state.update_mouse_clicked(false);
    }
    fn queue_fps(&mut self) {
        self.ui.queue_text(Section {
            text: &format!("FPS: {}", self.main_state.time_menager.fps()),
            color: [1.0, 1.0, 1.0, 1.0],
            screen_position: (0.0, 5.0),
            scale: wgpu_glyph::Scale::uniform(20.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Left,
                v_align: wgpu_glyph::VerticalAlign::Top,
            },
            ..Default::default()
        });
    }
    fn clear(&mut self, frame: &wgpu::SwapChainOutput) {
        self.gpu
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
    }
}

async fn main_async() {
    let event_loop = EventLoop::new();

    let builder = winit::window::WindowBuilder::new().with_title("Neothesia");
    let (window, gpu) = Window::new(builder, (1080, 720), &event_loop).await;
    let mut app = App::new(gpu, window);
    app.resize();
    app.gpu.submit();

    // Commented out control_flow stuff is related to:
    // https://github.com/gfx-rs/wgpu-rs/pull/306
    // I think it messes with my framerate so for now it's commented out, needs more testing

    // #[cfg(not(target_arch = "wasm32"))]
    // let mut last_update_inst = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // *control_flow = {
        //     #[cfg(not(target_arch = "wasm32"))]
        //     {
        //         ControlFlow::WaitUntil(
        //             std::time::Instant::now() + std::time::Duration::from_millis(10),
        //         )
        //     }
        //     #[cfg(target_arch = "wasm32")]
        //     {
        //         ControlFlow::Poll
        //     }
        // };
        match &event {
            Event::MainEventsCleared => {
                // #[cfg(not(target_arch = "wasm32"))]
                // {
                //     if last_update_inst.elapsed() > std::time::Duration::from_millis(20) {
                //         app.window.request_redraw();
                //         last_update_inst = std::time::Instant::now();
                //     }
                // }

                // #[cfg(target_arch = "wasm32")]
                app.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.event(AppEvent::WindowEvent(event, control_flow));
            }
            Event::RedrawRequested(_) => {
                app.update();
                app.render();
            }
            _ => {}
        }
    });
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        futures::executor::block_on(main_async());
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        wasm_bindgen_futures::spawn_local(main_async());
    }
}
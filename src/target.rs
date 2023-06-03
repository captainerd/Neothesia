use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::input_manager::InputManager;
use crate::text_renderer::TextRenderer;
use crate::utils::window::WindowState;
use crate::{output_manager::OutputManager, NeothesiaEvent, TransformUniform};
use wgpu_jumpstart::{Gpu, Uniform};
use winit::event_loop::EventLoopProxy;

use crate::iced_utils::IcedManager;
use winit::window::Window;

pub struct Target {
    pub window: Window,
    pub iced_manager: IcedManager,

    pub window_state: WindowState,
    pub gpu: Gpu,

    pub transform_uniform: Uniform<TransformUniform>,

    pub text_renderer: TextRenderer,

    pub output_manager: Rc<RefCell<OutputManager>>,
    pub input_manager: InputManager,
    pub midi_file: Option<Rc<lib_midi::Midi>>,
    pub config: Config,

    pub proxy: EventLoopProxy<NeothesiaEvent>,
}

impl Target {
    pub fn new(
        window: Window,
        window_state: WindowState,
        proxy: EventLoopProxy<NeothesiaEvent>,
        gpu: Gpu,
    ) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let text_renderer = TextRenderer::new(&gpu);

        let iced_manager = IcedManager::new(
            &gpu.device,
            &gpu.queue,
            gpu.texture_format,
            (
                window_state.physical_size.width,
                window_state.physical_size.height,
            ),
            window_state.scale_factor,
        );

        let args: Vec<String> = std::env::args().collect();

        let midi_file = if args.len() > 1 {
            if let Ok(midi) = lib_midi::Midi::new(&args[1]) {
                Some(Rc::new(midi))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            window,
            iced_manager,

            window_state,
            gpu,
            transform_uniform,

            text_renderer,

            output_manager: Default::default(),
            input_manager: InputManager::new(proxy.clone()),
            midi_file,
            config: Config::new(),
            proxy,
        }
    }

    pub fn resize(&mut self) {
        self.transform_uniform.data.update(
            self.window_state.logical_size.width,
            self.window_state.logical_size.height,
            self.window_state.scale_factor as f32,
        );
        self.transform_uniform.update(&self.gpu.queue);

        self.iced_manager.resize(
            (
                self.window_state.physical_size.width,
                self.window_state.physical_size.height,
            ),
            self.window_state.scale_factor,
        );
    }
}

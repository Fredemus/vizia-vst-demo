#[macro_use]
extern crate vst;

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;
use baseview::WindowHandle;

use std::sync::Arc;

use vizia::*;

mod ui;
pub use ui::*;

mod dsp;
use dsp::*;

struct GainPluginEditor {
    params: Arc<GainEffectParameters>,
    is_open: bool,
    // We need to keep track of the WindowHandle to close it correctly
    handle: Option<WindowHandle>
}

impl Editor for GainPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (300, 300)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        if self.is_open {
            return false;
        }

        self.is_open = true;

        let params = self.params.clone();

        let window_description = WindowDescription::new()
            .with_inner_size(300, 300)
            .with_title("Hello Plugin");

        let handle = Application::new(window_description, move |cx| {

            plugin_gui(cx, params.clone());
    
        }).open_parented(&ParentWindow(parent));

        self.handle = Some(handle);

        true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
        // If the host calls close on the editor, the editor needs to call close on its window_handle
        if let Some(mut handle) = self.handle.take() {
            handle.close();
        }
    }
}

struct GainPlugin {
    params: Arc<GainEffectParameters>,
    // editor: Option<GainPluginEditor>,
}

impl Default for GainPlugin {
    fn default() -> Self {
        let params = Arc::new(GainEffectParameters::default());
        Self {
            params: params.clone(),
            
        }
    }
}

impl Plugin for GainPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Vizia Gain Effect in Rust".to_string(),
            vendor: "Geom3trik".to_string(),
            unique_id: 243213073,
            version: 1,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: 1,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        let log_folder = ::dirs::home_dir().unwrap().join("tmp");

        let _ = ::std::fs::create_dir(log_folder.clone());

        let log_file = ::std::fs::File::create(log_folder.join("vizia_vst_demo.log")).unwrap();

        let log_config = ::simplelog::ConfigBuilder::new()
            .set_time_to_local(true)
            .build();

        let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file);

        ::log_panics::init();

        ::log::info!("init");
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        // Construct the editor here since it isn't Send
        Some(Box::new(GainPluginEditor {
            params: self.params.clone(),
            is_open: false,
            handle: None,
        }))
    }

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // Read the amplitude from the parameter object
        let amplitude = self.params.amplitude.get();
        // First, we destructure our audio buffer into an arbitrary number of
        // input and output buffers.  Usually, we'll be dealing with stereo (2 of each)
        // but that might change.
        for (input_buffer, output_buffer) in buffer.zip() {
            // Next, we'll loop through each individual sample so we can apply the amplitude
            // value to it.
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = *input_sample * amplitude;
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}



plugin_main!(GainPlugin);
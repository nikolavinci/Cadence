use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::sync::mpsc::Sender;

pub struct AudioCaptureEngine {
    _input_stream: Option<cpal::Stream>,
    _output_stream: Option<cpal::Stream>,
}

unsafe impl Send for AudioCaptureEngine {}
unsafe impl Sync for AudioCaptureEngine {}

impl AudioCaptureEngine {
    pub fn new() -> Self {
        Self {
            _input_stream: None,
            _output_stream: None,
        }
    }

    pub fn start(&mut self, tx: Sender<Vec<f32>>) -> Result<(), String> {
        let host = cpal::default_host();
        
        let input_device = host.default_input_device()
            .ok_or("No default input device found")?;
            
        let input_config = input_device.default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;
            
        println!("Starting WASAPI audio capture on: {}", input_device.name().unwrap_or_default());
        
        let tx_clone = tx.clone();
        
        let input_stream = match input_config.sample_format() {
            SampleFormat::F32 => Self::build_stream::<f32>(&input_device, &input_config.into(), tx_clone)?,
            SampleFormat::I16 => Self::build_stream::<i16>(&input_device, &input_config.into(), tx_clone)?,
            SampleFormat::U16 => Self::build_stream::<u16>(&input_device, &input_config.into(), tx_clone)?,
            _ => return Err("Unsupported sample format".into()),
        };
        
        input_stream.play().map_err(|e| format!("Failed to play input stream: {}", e))?;
        
        self._input_stream = Some(input_stream);
        
        Ok(())
    }
    
    fn build_stream<T>(
        device: &cpal::Device,
        config: &StreamConfig,
        tx: Sender<Vec<f32>>,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
        f32: cpal::FromSample<T>,
    {
        let channels = config.channels as usize;
        
        let err_fn = |err| eprintln!("Audio stream error: {}", err);
        
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                // Simplified channel mixer for Mono f32
                let mut output = Vec::with_capacity(data.len() / channels);
                for frame in data.chunks(channels) {
                    let mut sum: f32 = 0.0;
                    for sample in frame {
                        sum += sample.to_sample::<f32>();
                    }
                    output.push(sum / channels as f32);
                }
                
                // Mute check to avoid logging silence
                if output.len() > 0 && output.iter().any(|&x| x.abs() > 0.05) {
                     println!("🎤 Captured active audio frame, size: {}", output.len());
                }
                
                let _ = tx.send(output);
            },
            err_fn,
            None,
        ).map_err(|e| format!("Failed to build stream: {}", e))?;
        
        Ok(stream)
    }

    pub fn stop(&mut self) {
        self._input_stream = None;
        self._output_stream = None;
    }
}

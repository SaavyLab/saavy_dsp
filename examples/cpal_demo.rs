#[cfg(feature = "cpal-demo")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "cpal-demo")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[cfg(feature = "cpal-demo")]
use saavy_dsp::dsp_fluent::{
    envelope_node::{EnvelopeHandle, SharedAdsrEnvNode},
    node_extension::NodeExt,
    oscillator_node::OscNode,
    voice_node::RenderCtx,
};
#[cfg(feature = "cpal-demo")]
use crossterm::{event, terminal};
#[cfg(feature = "cpal-demo")]
use crossterm::event::{Event, KeyCode, KeyEventKind};
#[cfg(feature = "cpal-demo")]
use std::time::Duration;
#[cfg(feature = "cpal-demo")]
fn main() {
    if let Err(err) = run() {
        eprintln!("cpal demo error: {err}");
    }
}

#[cfg(feature = "cpal-demo")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device available")?;

    let supported_config = device.default_output_config()?;
    if supported_config.sample_format() != cpal::SampleFormat::F32 {
        return Err("cpal demo currently supports only f32 output".into());
    }

    let mut stream_config = supported_config.config();
    let block_size = 256u32;
    stream_config.buffer_size = cpal::BufferSize::Fixed(block_size);

    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate.0;

    let state = Arc::new(Mutex::new(EngineState::new(sample_rate, block_size, channels)));
    let callback_state = Arc::clone(&state);
    let control_state = Arc::clone(&state);

    let control_handle = std::thread::spawn(move || control_loop(control_state));

    let stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _| {
            use saavy_dsp::dsp_fluent::voice_node::VoiceNode;

            let mut guard = callback_state.lock().expect("engine mutex poisoned");
            let state = &mut *guard;

            let frames = if state.channels == 0 {
                0
            } else {
                data.len() / state.channels
            };

            if frames == 0 {
                data.fill(0.0);
                return;
            }

            if frames != state.buffer.len() {
                state.buffer.resize(frames, 0.0);
                state.ctx.block_size = frames;
            }

            state
                .synth
                .render_block(&mut state.ctx, &mut state.buffer);

            for frame in 0..frames {
                let sample = state.buffer[frame];
                for channel in 0..state.channels {
                    data[frame * state.channels + channel] = sample;
                }
            }
        },
        move |err| eprintln!("Stream error: {err}"),
        None,
    )?;

    stream.play()?;
    println!("Space: gate on/off, Esc: quit");
    let _ = control_handle.join();
    drop(stream);
    drop(state);
    Ok(())
}

#[cfg(feature = "cpal-demo")]
struct EngineState {
    synth: SynthChain,
    ctx: RenderCtx,
    buffer: Vec<f32>,
    channels: usize,
    gate: EnvelopeHandle,
    gate_on: bool,
}

#[cfg(feature = "cpal-demo")]
impl EngineState {
    fn new(sample_rate: u32, block_size: u32, channels: usize) -> Self {
        let channel_count = channels.max(1);
        let attack = 0.1;
        let decay = 0.1;
        let sustain = 0.8;
        let release = 0.3;
        let (env_node, gate) = SharedAdsrEnvNode::with_params(sample_rate as f32, attack, decay, sustain, release);
        let synth = OscNode::sine(220.0, sample_rate as f32).amplify(env_node);

        Self {
            synth,
            ctx: RenderCtx::new(sample_rate as f32, block_size as usize),
            buffer: vec![0.0; block_size as usize],
            channels: channel_count,
            gate,
            gate_on: false,
        }
    }
}

#[cfg(feature = "cpal-demo")]
type SynthChain = saavy_dsp::dsp_fluent::amplify::Amplify<OscNode, SharedAdsrEnvNode>;

#[cfg(feature = "cpal-demo")]
fn control_loop(state: Arc<Mutex<EngineState>>) {
    if terminal::enable_raw_mode().is_err() {
        eprintln!("failed to enable raw mode; controls disabled");
        return;
    }

    loop {
        if event::poll(Duration::from_millis(20)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    match key.code {
                        KeyCode::Char(' ') => {
                            let mut guard = state.lock().expect("engine mutex poisoned");
                            match key.kind {
                                KeyEventKind::Press => {
                                    if guard.gate_on {
                                        guard.gate.note_off();
                                        guard.gate_on = false;
                                    } else {
                                        guard.gate.note_on();
                                        guard.gate_on = true;
                                    }
                                }
                                KeyEventKind::Release => {
                                    if guard.gate_on {
                                        guard.gate.note_off();
                                        guard.gate_on = false;
                                    }
                                }
                                KeyEventKind::Repeat => {}
                            }
                        }
                        KeyCode::Esc => {
                            let mut guard = state.lock().expect("engine mutex poisoned");
                            if guard.gate_on {
                                guard.gate.note_off();
                                guard.gate_on = false;
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Mouse(_)) | Ok(Event::Resize(_, _)) => {},
                Ok(_) => break,
                Err(_) => break,
            }
        }
    }

    let _ = terminal::disable_raw_mode();
    println!("\nExiting...");
}

#[cfg(not(feature = "cpal-demo"))]
fn main() {
    eprintln!("Build with --features cpal-demo to run this example.");
}

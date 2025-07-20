use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{
    keyboard::{Event as KeyEvent, Key},
    widget::{button, column, slider},
    Element, Event, Subscription,
};
use num_traits::ToPrimitive;
use rust_decimal::{dec, Decimal};

fn main() -> iced::Result {
    iced::application("Stardust", App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| {
            let app = App::default();
            start_audio(app.state.clone());
            (app, iced::Task::none())
        })
}

#[derive(Debug, Default)]
struct App {
    state: Arc<Mutex<State>>,
}

#[derive(Debug, Default)]
struct State {
    active_notes: HashSet<Decimal>,
    wave: f32,
}

impl App {
    fn update(&mut self, message: Message) {
        let mut state = self.state.lock().unwrap();
        match message {
            Message::KeyPressed(key) => {
                if let Some(freq) = key_to_freq(key) {
                    state.active_notes.insert(freq);
                }
            }
            Message::KeyReleased(key) => {
                if let Some(freq) = key_to_freq(key) {
                    state.active_notes.remove(&freq);
                }
            }
            Message::SliderChanged(value) => state.wave = value,
            Message::None => {}
        }
    }
    fn view(&'_ self) -> Element<'_, Message> {
        let state = self.state.lock().unwrap();
        column![
            "Use o teclado QWERTY para tocar notas!",
            "Teclas: A W S E D F T G Y H U J K",
            button("fuck"),
            slider(0.0..=100.0, state.wave, Message::SliderChanged)
        ]
        .into()
    }
    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| match event {
            Event::Keyboard(KeyEvent::KeyPressed { key, .. }) => Message::KeyPressed(key),
            Event::Keyboard(KeyEvent::KeyReleased { key, .. }) => Message::KeyReleased(key),
            _ => Message::None,
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(Key),
    KeyReleased(Key),
    SliderChanged(f32),
    None,
}

// Mapeia tecla para frequência da nota
fn key_to_freq(key: Key) -> Option<Decimal> {
    Some(match key {
        Key::Character(char) => match char.as_str() {
            "q" => dec!(261.63), // C4
            "2" => dec!(277.18), // C#4
            "w" => dec!(293.66), // D4
            "3" => dec!(311.13), // D#4
            "e" => dec!(329.63), // E4
            "r" => dec!(349.23), // F4
            "5" => dec!(369.99), // F#4
            "t" => dec!(392.00), // G4
            "6" => dec!(415.30), // G#4
            "y" => dec!(440.00), // A4
            "7" => dec!(466.16), // A#4
            "u" => dec!(493.88), // B4
            "i" => dec!(523.25), // C5
            _ => return None,
        },
        _ => return None,
    })
}

fn start_audio(state: Arc<Mutex<State>>) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0 as f32;

        let mut sample_clock = 0f32;

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let state = state.lock().unwrap();
                    let voices: Vec<_> = state.active_notes.iter().map(|freq| *freq).collect();

                    for sample in data.iter_mut() {
                        let mut acc = 0.0;
                        for freq in &voices {
                            // saw
                            let phase = (sample_clock * freq.to_f32().unwrap() / sample_rate) % 1.0;
                            acc += 2. * phase - 1.;
                            // sine
                            // acc += (sample_clock * freq.to_f32().unwrap() * 2.0 * std::f32::consts::PI / sample_rate)
                            //     .sin();
                        }

                        *sample = if voices.is_empty() {
                            0.0
                        } else {
                            acc / voices.len() as f32 * 0.2 // volume
                        };

                        sample_clock += 1.0;
                        if sample_clock >= sample_rate {
                            sample_clock = 0.0;
                        }
                    }
                },
                |err| eprintln!("audio error: {:?}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

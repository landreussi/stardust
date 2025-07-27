use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{
    keyboard::{Event as KeyEvent, Key},
    widget::{button, column, row},
    Element, Event, Subscription,
};

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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum WaveShape {
    #[default]
    Sine,
    Triangle,
    Saw,
    Square,
}

impl WaveShape {
    fn generate_sample(&self, sample_clock: f32, freq: f32, sample_rate: f32) -> f32 {
        match self {
            Self::Sine => (sample_clock * freq * 2.0 * std::f32::consts::PI / sample_rate).sin(),
            Self::Saw => {
                let phase = (sample_clock * freq / sample_rate) % 1.0;
                2. * phase - 1.
            }
            Self::Square => {
                let phase = (sample_clock * freq / sample_rate) % 1.0;
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Triangle => {
                let phase = (sample_clock * freq / sample_rate) % 1.0;
                4.0 * (phase - 0.5).abs() - 1.0
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
enum Note {
    C4,
    Csus4,
    D4,
    Dsus4,
    E4,
    F4,
    Fsus4,
    G4,
    Gsus4,
    A4,
    Asus4,
    B4,
    C5,
}

impl Note {
    fn freq(&self) -> f32 {
        match self {
            Self::C4 => 261.63,
            Self::Csus4 => 277.18,
            Self::D4 => 293.66,
            Self::Dsus4 => 311.13,
            Self::E4 => 329.63,
            Self::F4 => 349.23,
            Self::Fsus4 => 369.99,
            Self::G4 => 392.00,
            Self::Gsus4 => 415.30,
            Self::A4 => 440.00,
            Self::Asus4 => 466.16,
            Self::B4 => 493.88,
            Self::C5 => 523.25,
        }
    }
}

#[derive(Debug, Default)]
struct State {
    active_notes: HashSet<Note>,
    wave_shape: WaveShape,
}

impl App {
    fn update(&mut self, message: Message) {
        let mut state = self.state.lock().unwrap();
        match message {
            Message::KeyPressed(key) => {
                if let Ok(note) = key.try_into() {
                    state.active_notes.insert(note);
                }
            }
            Message::KeyReleased(key) => {
                if let Ok(ref note) = key.try_into() {
                    state.active_notes.remove(note);
                }
            }
            Message::SineSelected => state.wave_shape = WaveShape::Sine,
            Message::SawSelected => state.wave_shape = WaveShape::Saw,
            Message::TriangleSelected => state.wave_shape = WaveShape::Triangle,
            Message::SquareSelected => state.wave_shape = WaveShape::Square,
            Message::None => {}
        }
    }
    fn view(&'_ self) -> Element<'_, Message> {
        column![row![
            button("Sine").on_press(Message::SineSelected),
            button("Saw").on_press(Message::SawSelected),
            button("Triangle").on_press(Message::TriangleSelected),
            button("Square").on_press(Message::SquareSelected),
        ]]
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
    SineSelected,
    SawSelected,
    TriangleSelected,
    SquareSelected,
    None,
}

impl TryFrom<Key> for Note {
    // This is fine once we'll remap this result to an option.
    type Error = ();

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        Ok(match key {
            Key::Character(char) => match char.as_str() {
                "q" => Self::C4,
                "2" => Self::Csus4,
                "w" => Self::D4,
                "3" => Self::Dsus4,
                "e" => Self::E4,
                "r" => Self::F4,
                "5" => Self::Fsus4,
                "t" => Self::G4,
                "6" => Self::Gsus4,
                "y" => Self::A4,
                "7" => Self::Asus4,
                "u" => Self::B4,
                "i" => Self::C5,
                _ => return Err(()),
            },
            _ => return Err(()),
        })
    }
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
                    let notes: Vec<_> = state.active_notes.iter().collect();

                    for sample in data.iter_mut() {
                        let mut acc = 0.0;
                        for note in &notes {
                            acc += state.wave_shape.generate_sample(
                                sample_clock,
                                note.freq(),
                                sample_rate,
                            );
                        }

                        *sample = if notes.is_empty() {
                            0.0
                        } else {
                            acc / notes.len() as f32 * 0.2 // volume
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

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{
    keyboard::{Event as KeyEvent, Key},
    mouse,
    widget::{
        button, canvas,
        canvas::{Frame, Geometry, Path, Program, Stroke},
        column, image, row,
    },
    Color, Element, Event, Point, Renderer, Size, Subscription, Theme,
};
use strum::{Display, EnumIter, IntoEnumIterator};

fn main() -> iced::Result {
    iced::application("Stardust", App::update, App::view)
        .theme(|_| Theme::Dracula)
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

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, EnumIter, Display)]
enum Note {
    C3,
    CSharp3,
    D3,
    DSharp3,
    E3,
    F3,
    FSharp3,
    G3,
    GSharp3,
    A3,
    ASharp3,
    B3,
    C4,
    CSharp4,
    D4,
    DSharp4,
    E4,
    F4,
    FSharp4,
    G4,
    GSharp4,
    A4,
    ASharp4,
    B4,
    C5,
    CSharp5,
    D5,
    DSharp5,
    E5,
    F5,
}

impl Note {
    fn freq(&self) -> f32 {
        match self {
            Self::C3 => 130.81,
            Self::CSharp3 => 138.59,
            Self::D3 => 146.83,
            Self::DSharp3 => 155.56,
            Self::E3 => 164.81,
            Self::F3 => 174.61,
            Self::FSharp3 => 185.00,
            Self::G3 => 196.00,
            Self::GSharp3 => 207.65,
            Self::A3 => 220.00,
            Self::ASharp3 => 233.08,
            Self::B3 => 246.94,
            Self::C4 => 261.63,
            Self::CSharp4 => 277.18,
            Self::D4 => 293.66,
            Self::DSharp4 => 311.13,
            Self::E4 => 329.63,
            Self::F4 => 349.23,
            Self::FSharp4 => 369.99,
            Self::G4 => 392.00,
            Self::GSharp4 => 415.30,
            Self::A4 => 440.00,
            Self::ASharp4 => 466.16,
            Self::B4 => 493.88,
            Self::C5 => 523.25,
            Self::CSharp5 => 554.37,
            Self::D5 => 587.33,
            Self::DSharp5 => 622.25,
            Self::E5 => 659.25,
            Self::F5 => 698.46,
        }
    }

    fn major_notes() -> impl Iterator<Item = Self> {
        let is_major = |note: &Self| !note.to_string().contains("Sharp");
        Self::iter().filter(is_major)
    }
}

#[derive(Debug, Default)]
struct State {
    active_notes: HashSet<Note>,
    wave_shape: WaveShape,
}

struct Piano;

impl<Message> Program<Message> for Piano {
    type State = State;
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(
            renderer,
            Size {
                width: 4000.,
                height: 150.,
            },
        );
        let white_key_width = 30.;
        let white_key_height = frame.height();
        let black_key_width = 15.;
        let black_key_height = white_key_height * 0.6;

        let white_keys = Note::major_notes().count();
        let black_key_indices = [0, 1, 3, 4, 5, 7, 8]; // Relative positions in octave

        // Draw white keys
        for i in 0..white_keys {
            let x = i as f32 * white_key_width;
            let rect = Path::rectangle(
                Point::new(x, 0.0),
                Size::new(white_key_width, white_key_height),
            );
            frame.fill(&rect, Color::WHITE);
            frame.stroke(&rect, Stroke::default().with_color(Color::BLACK));
        }

        // Draw black keys (except where there's no black key)
        for i in 0..2 {
            for &pos in &black_key_indices {
                let x = ((i * 7 + pos) as f32 + 1.0) * white_key_width - black_key_width / 2.0;
                let rect = Path::rectangle(
                    Point::new(x, 0.0),
                    Size::new(black_key_width, black_key_height),
                );
                frame.fill(&rect, Color::BLACK);
            }
        }

        vec![frame.into_geometry()]
    }
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
        column![
            image("stardust.png"),
            row![
                button("Sine").on_press(Message::SineSelected),
                button("Saw").on_press(Message::SawSelected),
                button("Triangle").on_press(Message::TriangleSelected),
                button("Square").on_press(Message::SquareSelected),
            ],
            canvas(Piano)
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
    SineSelected,
    SawSelected,
    TriangleSelected,
    SquareSelected,
    None,
}

impl TryFrom<Key> for Note {
    // This is fine once we'll ignore the error.
    type Error = ();

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        Ok(match key {
            Key::Character(char) => match char.as_str() {
                "z" => Self::C3,
                "s" => Self::CSharp3,
                "x" => Self::D3,
                "d" => Self::DSharp3,
                "c" => Self::E3,
                "v" => Self::F3,
                "g" => Self::FSharp3,
                "b" => Self::G3,
                "h" => Self::GSharp3,
                "n" => Self::A3,
                "j" => Self::ASharp3,
                "m" => Self::B3,
                "q" => Self::C4,
                "2" => Self::CSharp4,
                "w" => Self::D4,
                "3" => Self::DSharp4,
                "e" => Self::E4,
                "r" => Self::F4,
                "5" => Self::FSharp4,
                "t" => Self::G4,
                "6" => Self::GSharp4,
                "y" => Self::A4,
                "7" => Self::ASharp4,
                "u" => Self::B4,
                "i" => Self::C5,
                "9" => Self::CSharp5,
                "o" => Self::D5,
                "0" => Self::DSharp5,
                "p" => Self::E5,
                "[" => Self::F5,
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

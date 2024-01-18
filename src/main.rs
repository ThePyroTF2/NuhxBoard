// use regex::Regex;
// use std::io::{self, BufRead};
// use std::process::Command;
//
// fn main() -> io::Result<()> {
//     let mut child = Command::new("xinput")
//         .arg("test-xi2")
//         .arg("--root")
//         .stdout(std::process::Stdio::piped())
//         .spawn()?;
//
//     let reader = io::BufReader::new(child.stdout.take().unwrap());
//
//     let mut key_down = false;
//     let mut key_up = false;
//
//     for line in reader.lines() {
//         let line = line?;
//
//         if line.contains("EVENT type 2") {
//             key_down = true;
//         } else if line.contains("EVENT type 3") {
//             key_up = true;
//         } else if line.contains("detail:") {
//             let re = Regex::new(r"detail:\s*(\d+)").unwrap();
//
//             if key_down {
//                 let captures = re.captures(&line).unwrap();
//                 let keycode: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
//                 println!("Key down: {}", keycode)
//             }
//             if key_up {
//                 let captures = re.captures(&line).unwrap();
//                 let keycode: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
//                 println!("Key up: {}", keycode)
//             }
//
//             key_down = false;
//             key_up = false;
//         }
//     }
//
//     Ok(())
// }
#![allow(unused)]
mod config;
mod style;
use clap::Parser;
use colors_transform::{AlphaColor, Color as ColorTrait, Rgb};
use config::*;
use iced::{
    keyboard, mouse,
    widget::{
        canvas,
        canvas::{Cache, Geometry, Path},
        container,
    },
    Application, Color, Command, Event, Length, Point, Rectangle, Renderer, Subscription, Theme,
};
use serde::Deserialize;
use std::{fs::File, io::prelude::*};
use style::*;

struct NuhxBoard {
    config: Config,
    style: Style,
    canvas: Cache,
    pressed_keys: Vec<u32>,
}

#[derive(Debug)]
enum Message {
    KeyDown(u32),
    KeyUp(u32),
}

#[derive(Default)]
struct Flags {
    config: Config,
    style: Style,
}

impl Application for NuhxBoard {
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Flags) -> (Self, Command<Message>) {
        (
            Self {
                config: flags.config,
                style: flags.style,
                canvas: Cache::default(),
                pressed_keys: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("NuhxBoard")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);

        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        let red: f32 = Into::<f32>::into(self.style.background_color.red) / 255.0;
        let green: f32 = Into::<f32>::into(self.style.background_color.green) / 255.0;
        let blue: f32 = Into::<f32>::into(self.style.background_color.blue) / 255.0;
        let palette: iced::theme::Palette = iced::theme::Palette {
            background: Color::from_rgb(red, green, blue),
            ..iced::theme::Palette::DARK
        };
        Theme::Custom(Box::new(iced::theme::Custom::new(palette)))
    }
}

impl<Message> canvas::Program<Message, Renderer> for NuhxBoard {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let canvas = self.canvas.draw(renderer, bounds.size(), |frame| {
            for element in &self.config.elements {
                match element {
                    BoardElement::KeyboardKey(def) => {
                        let mut boundaries_iter = def.boundaries.iter();
                        let key = Path::new(|builder| {
                            builder.move_to((*boundaries_iter.next().unwrap()).clone().into());
                            for boundary in boundaries_iter {
                                builder.line_to((*boundary).clone().into());
                            }
                            builder.close()
                        });

                        // TODO: Avoid creating a new ElementStyle to contain the default key style
                        // We should end up with an element_style that only points to the app's
                        // state such that we can be sure the data will last as long as the app.
                        // Doing what we do now forces us to leak the font family string. Not
                        // necessarily a problem, but there are far more elegant ways to handle
                        // this data.
                        let default_key_style = ElementStyle {
                            key: def.id,
                            value: style::ElementStyleUnion::KeyStyle(
                                self.style.default_key_style.clone(),
                            ),
                        };
                        let element_style = match &self
                            .style
                            .element_styles
                            .iter()
                            .find(|style| style.key == def.id)
                            .unwrap_or(&default_key_style)
                            .value
                        {
                            ElementStyleUnion::KeyStyle(style) => style,
                            _ => unreachable!(),
                        };

                        let fill_color = match self.pressed_keys.contains(&def.id) {
                            true => &element_style.pressed.background,
                            false => &element_style.loose.background,
                        };
                        frame.fill(
                            &key,
                            Color::from_rgb(
                                fill_color.red.into(),
                                fill_color.green.into(),
                                fill_color.blue.into(),
                            ),
                        );
                        frame.fill_text(canvas::Text {
                            content: def.text.clone(),
                            position: def.text_position.clone().into(),
                            color: match self.pressed_keys.contains(&def.id) {
                                true => Color::from_rgb(
                                    element_style.pressed.text.red.into(),
                                    element_style.pressed.text.green.into(),
                                    element_style.pressed.text.blue.into(),
                                ),
                                false => Color::from_rgb(
                                    element_style.loose.text.red.into(),
                                    element_style.loose.text.green.into(),
                                    element_style.loose.text.blue.into(),
                                ),
                            },
                            size: element_style.loose.font.size,
                            font: iced::Font {
                                family: iced::font::Family::Name(
                                    match self.pressed_keys.contains(&def.id) {
                                        true => {
                                            element_style.pressed.font.font_family.clone().leak()
                                        }
                                        false => {
                                            element_style.loose.font.font_family.clone().leak()
                                        }
                                    },
                                ),
                                weight: match self.pressed_keys.contains(&def.id) {
                                    true => {
                                        if element_style.pressed.font.style & 0b00000001 > 0 {
                                            iced::font::Weight::Bold
                                        } else {
                                            iced::font::Weight::Normal
                                        }
                                    }
                                    false => {
                                        if element_style.loose.font.style & 0b00000001 > 0 {
                                            iced::font::Weight::Bold
                                        } else {
                                            iced::font::Weight::Normal
                                        }
                                    }
                                },
                                stretch: match self.pressed_keys.contains(&def.id) {
                                    true => {
                                        if element_style.pressed.font.style & 0b00000010 > 0 {
                                            iced::font::Stretch::Expanded
                                        } else {
                                            iced::font::Stretch::Normal
                                        }
                                    }
                                    false => {
                                        if element_style.loose.font.style & 0b00000010 > 0 {
                                            iced::font::Stretch::Expanded
                                        } else {
                                            iced::font::Stretch::Normal
                                        }
                                    }
                                },
                                monospaced: false,
                            },
                            ..canvas::Text::default()
                        })
                    }
                    _ => unimplemented!(),
                }
            }
        });
        vec![canvas]
    }
}

#[derive(Parser, Debug)]
#[command(author = "justDeeevin", version = "0.1.0")]
struct Args {
    #[arg(short, long)]
    config_path: String,

    #[arg(short, long)]
    style_path: String,
}

fn main() -> iced::Result {
    let args = Args::parse();

    let mut config_file = match File::open(&args.config_path) {
        Err(why) => panic!(
            "Error opening config file (given path: {}): {}",
            args.config_path, why
        ),
        Ok(file) => file,
    };
    let mut config_string = String::new();
    if let Err(why) = config_file.read_to_string(&mut config_string) {
        panic!("Error reading config file: {}", why)
    };
    let config: Config = match serde_json::from_str(&config_string) {
        Err(why) => panic!("Error parsing config file: {}", why),
        Ok(config) => config,
    };

    let mut style_file = match File::open(&args.style_path) {
        Err(why) => panic!(
            "Error opening style file (given path: {}): {}",
            args.style_path, why
        ),
        Ok(file) => file,
    };
    let mut style_string = String::new();
    if let Err(why) = style_file.read_to_string(&mut style_string) {
        panic!("Error reading style file: {}", why)
    };
    let style: Style = match serde_json::from_str(&style_string) {
        Err(why) => panic!("Error parsing style file: {}", why),
        Ok(style) => style,
    };

    let icon = iced::window::icon::from_file(std::path::Path::new("NuhxBoard.png")).unwrap();

    let flags = Flags { config, style };
    let settings = iced::Settings {
        window: iced::window::Settings {
            size: (flags.config.width, flags.config.height),
            resizable: true,
            icon: Some(icon),
            ..iced::window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings)
}

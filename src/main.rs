use eframe::{
    egui::{self, Key, Margin, TextStyle},
    epaint::{FontFamily, FontId},
};
use std::{env, fs, path::PathBuf, process};

// presentation is a bunch of slides
enum Slide {
    Paragraph(String),
    Image(String),
}

fn main() {
    // CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage {}", args[0]);
        process::exit(1);
    }

    // input file
    let path = &args[1];
    let contents = fs::read_to_string(path).expect("failed to read");

    // parse into a presentation
    let presentation: Vec<Slide> = contents
        .trim()
        .split("\n\n")
        .map(|line| {
            if line.starts_with("!") {
                let path = PathBuf::from(line.strip_prefix("!").unwrap());
                let absolute_path = fs::canonicalize(&path).unwrap();
                Slide::Image(format!("file://{}", absolute_path.to_str().unwrap()))
            } else {
                Slide::Paragraph(line.to_string())
            }
        })
        .collect();

    // egui stuff
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "rent",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, presentation))),
    )
    .expect("can not run the app");
}

#[derive(Default)]
struct App {
    presentation: Vec<Slide>,
    current: usize,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, presentation: Vec<Slide>) -> Self {
        Self {
            presentation,
            current: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        // current slide
        let slide = self.presentation.get(self.current).unwrap();

        // user input
        if ctx.input(|i| i.key_pressed(Key::H)) {
            if self.current > 0 {
                self.current -= 1;
            }
        }
        if ctx.input(|i| i.key_pressed(Key::L)) {
            if self.current < self.presentation.len() - 1 {
                self.current += 1;
            }
        }

        // display
        egui::CentralPanel::default().show(ctx, |ui| match slide {
            Slide::Paragraph(text) => {
                ui.centered_and_justified(|ui| {
                    // get the longest line
                    let longest = text
                        .split("\n")
                        .reduce(|a, b| if a.len() < b.len() { b } else { a })
                        .unwrap()
                        .len();

                    // get window width
                    let width =
                        ctx.input(|i| i.viewport().inner_rect.map(|r| r.width()).unwrap_or(0.0));

                    // styles
                    ctx.style_mut(|s| {
                        // calculate font size
                        let upper_limit = width * 0.025;
                        let font_size = width / 3.0 * (8.0 / longest as f32);
                        let font_size = if font_size > upper_limit {
                            upper_limit
                        } else {
                            font_size
                        };

                        // change font size
                        let t = s.text_styles.iter_mut().find(|f| *f.0 == TextStyle::Body);
                        if let Some(t) = t {
                            *t.1 = FontId::new(font_size, FontFamily::Proportional);
                        }

                        // calculate window margin
                        let margin = width * 0.01;
                        s.spacing.window_margin = Margin::symmetric(margin, margin);
                    });

                    // display text
                    ui.label(text);
                });
            }
            Slide::Image(path) => {
                ui.centered_and_justified(|ui| {
                    ui.image(path);
                });
            }
        });
    }
}

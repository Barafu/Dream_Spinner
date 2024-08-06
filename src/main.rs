#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::{bail, Result};
use std::sync::{Arc, RwLock};
use dream_spinner::app_settings::Settings;


//Parsing CLI arguments

#[derive(Debug, PartialEq)]
struct ParsedArguments {
    command: MainCommand,
    handle: Option<usize>,
}

#[derive(Debug, PartialEq)]
enum MainCommand {
    Show,
    Preview,
    Config,
}

// Here goes the specification. We need to support one of the following arguments:
// /p:handle -- show the preview into given handle
// /s        -- show the main event fullscreen
// /s:handle -- show the main event into a given handle
// /c        -- show the configuration window
// /c:handle -- no idea
//
// Now, the trouble part. The arguments may be either in a lower or upper case.
// The handle may be separated by either ":" or space.
// Another day with the great standartisation by Micro$oft.

///Parses command line arfuments. Returns Err() if arguments are malformed.
fn parse_args(args_in: &[String]) -> Result<ParsedArguments> {
    //Lowercasing everything:
    let args: Vec<String> = args_in.iter().map(|s| s.to_lowercase()).collect();

    //If parsing goes wrong, just return Err;
    //With no args or wrong amount of args, abort parsing.
    if args.len() > 3 || args.is_empty() {
        bail!("Wrong number of arguments");
    }
    //No arguments at all is acceptable
    if args.len() == 1 {
        return Ok(ParsedArguments {
            command: MainCommand::Show,
            handle: None,
        });
    }

    //If argument 2 exists, parse it to handle
    let handle1: Option<usize> = match args.get(2) {
        Some(s) => match s.parse() {
            Ok(0_usize) | Err(_) => bail!("Can't parse argument 2"),
            Ok(n) => Some(n),
        },
        None => None,
    };

    //Parse argument 1
    let command_chars: Vec<char> = match args.get(1) {
        Some(s) => s.chars().collect(),
        None => bail!("Can't prepare argument 1"),
    };

    if command_chars.is_empty() || command_chars[0] != '/' {
        bail!("Argument 1 bad form: {command_chars:?}");
    }
    //Check if argument 1 has a handle provided after :
    let handle2;
    if command_chars.len() > 2 {
        if command_chars[2] != ':' {
            bail!("Argument 1 bad form: {command_chars:?}");
        } else {
            let s: String = command_chars[3..].iter().collect();
            handle2 = match s.parse() {
                Ok(0_usize) | Err(_) => {
                    bail!("Can't parse handle: {command_chars:?}")
                }
                Ok(n) => Some(n),
            }
        }
    } else {
        handle2 = None;
    }
    //If both handles are set, it is a parse error
    if handle1.is_some() && handle2.is_some() {
        bail!("Provided 2 handles together");
    }
    let handle = handle1.or(handle2);
    //Parse second letter of arg1 to determine command
    let command = match command_chars[1] {
        'c' => MainCommand::Config,
        's' => MainCommand::Show,
        'p' => MainCommand::Preview,
        _ => bail!("Unknown command letter"),
    };

    //Preview should not be passed without a handle
    if let MainCommand::Preview = command {
        if handle.is_none() {
            bail!("Preview should not be passed with a handle");
        }
    }

    Ok(ParsedArguments { command, handle })
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let args: Vec<String> = std::env::args().collect();
    let parsed = parse_args(&args).unwrap();


    // Display dreams
    if parsed.command == MainCommand::Show {    

        // DreamSpinner supports multiple displays. In OS, therer are concepts of 
        // a primary display and secondary displays. In eframe, there are primary
        // window and secondary windows. Secondary windows have to be created from
        // the draw code of primary window.
        //
        // So, we detect primary monitor and create a primary window on it,
        // then we pass the list of remaining monitors to the primary window for
        // creating secondary windows.

        let settings = Settings::read_from_file_default().unwrap();
        let settings = Arc::new(RwLock::new(settings));

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                // .with_position([primary_display.x as f32, primary_display.y as f32])
                // .with_fullscreen(true)
                .with_taskbar(false)
                .with_drag_and_drop(false)
                .with_icon(
                    // NOTE: Adding an icon is optional
                    eframe::icon_data::from_png_bytes(
                        &include_bytes!("../assets/icon-256.png")[..],
                    )
                    .expect("Failed to load icon"),
                ),
            ..Default::default()
        };
        return eframe::run_native(
            "DreamSpinner",
            native_options,
            Box::new(|cc| Ok(Box::new(dream_spinner::DreamSpinner::new(cc, settings)))),
        );
    };
    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Ok(Box::new(dream_spinner::DreamSpinner::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

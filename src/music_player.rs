extern crate midir;
extern crate rustbox;

use std::error::Error;
use std;
use utils;
use self::rustbox::{RustBox, Event};

fn play_music(midi_out: &mut midir::MidiOutputConnection, event: &utils::MusicEvent)
{
    for message in &event.midi_messages {
        if let Err(e) = midi_out.send(&message) {
            println!("Error occured while playing some event: {}", e.description());
            if let Some(e) = e.cause() {
                println!("{}", e.description());
            }
        }
    }
}

fn init_ref_pos(width: usize, height: usize) -> (usize, usize) {
    let keyboard_height = 8;
    let keyboard_width = 188;
    ((width - keyboard_width) / 2, (height - keyboard_height) / 2)
}

fn update_screen(ui: &RustBox, ref_x: usize, ref_y: usize)
{
    ui.clear();
    ui.print(ref_x, ref_y + 10, rustbox::RB_BOLD, rustbox::Color::Magenta, rustbox::Color::Default, "press <CTRL + q> to quit");
    ui.print(ref_x, ref_y + 11, rustbox::RB_BOLD, rustbox::Color::Magenta, rustbox::Color::Default, "press <space> to pause/unpause");
    ui.present();
}

pub fn play(song: utils::Song, midi_output_port: u32) {
    let midi_out = midir::MidiOutput::new("Midi output from pianoterm-rs");
    if let Err(e) = midi_out {
        println!("Error occured while initialising the midi output: {}", e.description());
        return ();
    }

    let midi_out = midi_out.unwrap();
    let conn_out = midi_out.connect(midi_output_port, "output midi port from pianoterm-rs");

    if let Err(e) = conn_out {
        println!("Failed to open midi output port: {}", e.kind().description());
        return ();
    }

    let mut conn_out = conn_out.unwrap();
    let ui = RustBox::init(Default::default());
    if let Err(e) = ui {
        println!("Failed to initialise the user interface (rustbox): {}", e.description());
        return ();
    };

    let ui = ui.unwrap();
    let (mut x, mut y) = init_ref_pos(ui.width(), ui.height());

    let nb_events = song.len();
    for i in 0 .. nb_events {

        update_screen(&ui, x, y);
        let current_event = &song[i];
        play_music(&mut conn_out, current_event);

        if i != nb_events - 1 {
            let time_to_wait = song[i + 1].time_in_ns - current_event.time_in_ns;
            let one_billion = 1_000_000_000;
            let time_to_wait = std::time::Duration::new(time_to_wait / one_billion, (time_to_wait % one_billion) as u32);

            let started_time = std::time::Instant::now();

            loop {
                let time_now = std::time::Instant::now();
                let waited_time = time_now - started_time;
                if waited_time > time_to_wait {
                    break;
                }

                let time_to_sleep = {
                    let remaining_time = time_to_wait - waited_time;
                    if remaining_time > std::time::Duration::from_millis(100) {
                        std::time::Duration::from_millis(100)
                    } else {
                        remaining_time
                    }
                };


                match ui.peek_event(time_to_sleep, false) {
                    Ok(Event::NoEvent)
                    | Ok(Event::MouseEvent(_, _, _)) => (),
                    Ok(Event::KeyEventRaw(_, _, _)) => panic!("Raw event received, whereas the raw parameter to peek_event was set to false!"),
                    Ok(Event::ResizeEvent(w, h)) => {
                        if (w < 0) || (h < 0) {
                            panic!("new window size has negtive components. Can't happen after a successful init!");
                        }
                        let (this_x, this_y) = init_ref_pos(w as usize, h as usize);
                        x = this_x;
                        y = this_y;
                    },
                    Ok(Event::KeyEvent(_)) => (), // TODO handle key events.
                    Err(e) => { println!("Error occured in rustbox: {}", e.description()); return (); },
                };

            }
        }
    }
}

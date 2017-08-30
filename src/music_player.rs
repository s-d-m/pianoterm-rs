extern crate midir;

use std::error::Error;
use std;
use utils;

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

    let nb_events = song.len();
    for i in 0 .. nb_events {
        println!("playing event {} out of {}", i, nb_events);
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
                std::thread::sleep(time_to_sleep);
            }
        }
    }
}

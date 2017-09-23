extern crate clap;

use std::str::FromStr;
use std::error::Error;

mod ports_printer;
mod midi_reader;
mod keyboard_events_extractor;
mod utils;
mod music_player;
mod signal_handler;

fn main() {
    let input_midi_port_option_name = "input port";
    let input_midi_file_option_name = "input midi file";
    let output_midi_port_option_name = "output port";
    let list_option_name = "list";

    let options = clap::App::new("pianoterm-rs")
        .version("0.1")
        .author("Samuel DA MOTA <da.mota.sam@gmail.com>")
        .about("Displays a keyboard in your terminal")
        .arg(clap::Arg::with_name(input_midi_port_option_name)
                 .short("i")
                 .long("input-port")
                 .help("The midi input port to listen to (use --list to list them)")
                 .takes_value(true)
                 .value_name("INPUT_PORT_NUMBER")
                 .required_unless_one(&[input_midi_file_option_name, list_option_name])
                 .conflicts_with(input_midi_file_option_name))
        .arg(clap::Arg::with_name(output_midi_port_option_name)
                 .short("o")
                 .long("output-port")
                 .takes_value(true)
                 .value_name("OUTPUT_PORT_NUM")
                 .help("The midi output port to send music to")
                 .required_unless(list_option_name))
        .arg(clap::Arg::with_name(list_option_name)
                 .short("l")
                 .long("list")
                 .takes_value(false)
                 .help("lists the available midi port"))
        .arg(clap::Arg::with_name(input_midi_file_option_name)
                 .required_unless_one(&[list_option_name, input_midi_port_option_name]))
        .get_matches();

    if options.is_present(list_option_name) {
        ports_printer::print_inputs();
        println!();
        ports_printer::print_outputs();
        return;
    }

    let port = match options.value_of(output_midi_port_option_name) {
        Some(value) => {
            match u32::from_str(value) {
                Ok(v) => v,
                Err(e) => {
                    println!("Error: invalid output port given.{}\n. Below is the list of possible output ports", e.description());
                    ports_printer::print_outputs();
                    std::process::exit(2)
                }
            }
        }
        None => {
            println!("Error: an output port must be given. Below is the list of possible output ports");
            ports_printer::print_outputs();
            std::process::exit(2)
        }
    };

    signal_handler::register_signal_listener();

    match options.value_of(input_midi_file_option_name) {
        Some(filename) => {
            let midi_events = midi_reader::get_midi_events(filename).unwrap_or_else(|e| {
                println!("Error occured: {}", e);
                std::process::exit(2)
            });

            let keyboard_events = keyboard_events_extractor::get_key_events(&midi_events)
                .unwrap_or_else(|e| {
                                    println!("Error occured: {}", e);
                                    std::process::exit(2)
                                });

            println!("extracted {} keyboard events", keyboard_events.len());

            let song = utils::group_events_by_time(&midi_events, &keyboard_events).unwrap_or_else(|e| {
                println!("Error occured while grouping events occuring at the same time: {}", e);
                std::process::exit(2);
            });


            music_player::play(song, port);
        }
        None => {
            println!("listening to input port for midi events");
            let input_midi_port = match options.value_of(input_midi_port_option_name) {
                Some(value) => match u32::from_str(value) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Error: invalid input port given.{}\n. Below is the list of possible input ports", e.description());
                        ports_printer::print_inputs();
                        std::process::exit(2)
                    }
                },
                None => {
                    println!("Error should select either an input port or a midi file");
                    std::process::exit(2);
                },
            };

            music_player::play_midi_input(input_midi_port, port);
        }
    }
}

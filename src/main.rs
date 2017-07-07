extern crate clap;
extern crate midir;

// Todo find a way to factorise print_input_ports and print_output_ports.
// Maybe with a macro
fn print_input_ports() {
    // listing input midi ports
    let input = midir::MidiInput::new("");
    match input {
        Err(e) => { println!("Failed to list input ports: {}", e) }
        Ok(input) => {
            let nb_ports = input.port_count();
            match nb_ports {
                0 => { println!("Sorry: no input midi port found"); }
                1 => {
                    println!("! input port found:");
                    let name_res = input.port_name(0);
                    match name_res {
                        Ok(name) => println!("  0 -> {}", name),
                        Err(e) => println!("Error while retrieving port's name: {}", e),
                    }
                }
                _ => {
                    println!("{} input ports found:", nb_ports);
                    for i in 0..nb_ports {
                        let name_res = input.port_name(i);
                        match name_res {
                            Ok(name) => println!("  {} -> {}", i, name),
                            Err(e) => println!("Error while retrieving port {}'s name: {}", i, e),
                        }
                    }
                }
            }
        }
    }
}

fn print_output_ports() {
    // listing output midi ports
    let output = midir::MidiOutput::new("");
    match output {
        Err(e) => { println!("Failed to list output ports: {}", e) }
        Ok(output) => {
            let nb_ports = output.port_count();
            match nb_ports {
                0 => { println!("Sorry: no output midi port found"); }
                1 => {
                    println!("! output port found:");
                    let name_res = output.port_name(0);
                    match name_res {
                        Ok(name) => println!("  0 -> {}", name),
                        Err(e) => println!("Error while retrieving port's name: {}", e),
                    }
                }
                _ => {
                    println!("{} output ports found:", nb_ports);
                    for i in 0..nb_ports {
                        let name_res = output.port_name(i);
                        match name_res {
                            Ok(name) => println!("  {} -> {}", i, name),
                            Err(e) => println!("Error while retrieving port {}'s name: {}", i, e),
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let input_midi_port_option_name = "input port";
    let input_midi_file_option_name = "input midi file";
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
        .arg(clap::Arg::with_name("output-port")
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
        print_input_ports();
        println!();
        print_output_ports();
        return;
    }

}

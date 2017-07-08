extern crate clap;

mod ports_printer;

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
        ports_printer::print_inputs();
        println!();
        ports_printer::print_outputs();
        return;
    }

}

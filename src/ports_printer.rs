extern crate midir;


// Todo find a way to factorise print_input_ports and print_output_ports.
// Maybe with a macro
pub fn print_inputs() {
    // listing input midi ports
    let input = midir::MidiInput::new("");
    match input {
        Err(e) => println!("Failed to list input ports: {}", e),
        Ok(input) => {
            let nb_ports = input.port_count();
            match nb_ports {
                0 => {
                    println!("Sorry: no input midi port found");
                }
                1 => {
                    println!("1 input port found:");
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

pub fn print_outputs() {
    // listing output midi ports
    let output = midir::MidiOutput::new("");
    match output {
        Err(e) => println!("Failed to list output ports: {}", e),
        Ok(output) => {
            let nb_ports = output.port_count();
            match nb_ports {
                0 => {
                    println!("Sorry: no output midi port found");
                }
                1 => {
                    println!("1 output port found:");
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

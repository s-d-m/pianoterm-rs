use std::io::prelude::*;
use std::error::Error;
use std;

pub struct MidiEvent {
    // time: u64,
    // data: Vec<u8>,
}

fn read_big_endian_32(file: &mut std::fs::File) -> Option<u32>
{
    let mut value_buffer: [u8; 4] = [0; 4];
    if file.read_exact(&mut value_buffer).is_err()  {
        return None;
    }

    let res = ((value_buffer[0] as u32) << 24) |
        ((value_buffer[1] as u32) << 16) |
        ((value_buffer[2] as u32) << 8) |
        (value_buffer[3] as u32);

    return Some(res);
}


pub fn get_midi_events(filename: &str) -> Result<Vec<MidiEvent>, String> {
    let mut file = match std::fs::File::open(filename) {
        Err(e) => {
            return Err(format!("Failed to open file {}: {}", filename, e.description()));
        }
        Ok(f) => f,
    };

    {
        let mut header_buffer: [u8; 4] = [0; 4];
        if let Err(e) = file.read_exact(&mut header_buffer) {
            return Err(format!("Failed to read the header out of file {}: {}", filename, e.description()));
        }


        if (header_buffer[0] != 'M' as u8) || (header_buffer[1] != 'T' as u8) ||
            (header_buffer[2] != 'h' as u8) || (header_buffer[3] != 'd' as u8) {
            return Err(format!("The file {} doesn't start by the correct header", filename));
        }
    }

    {
      let header_size = read_big_endian_32(&mut file);
      match header_size {
          None => return Err("The file ".to_owned() + filename + " is too short"),
          Some(6) => (),
          Some(x) => return Err(format!("Invalid header size found in file {}. Expecting 6, got {}", filename, x)),
      }
    }

    return Ok(Vec::new());
}

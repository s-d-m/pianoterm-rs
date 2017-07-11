extern crate byteorder;
use midi_reader::byteorder::{ReadBytesExt, BigEndian};

use std::io::prelude::*;
use std::error::Error;
use std;

pub struct MidiEvent {
    // time: u64,
    // data: Vec<u8>,
}

fn read_magic_number(mut file: &mut std::fs::File, filename: &str) -> Result<(), String>
{
    let mut header_buffer: [u8; 4] = [0; 4];
    if let Err(e) = file.read_exact(&mut header_buffer) {
        return Err(format!("Failed to read the header out of file {}: {}", filename, e.description()));
    }


    if (header_buffer[0] != 'M' as u8) || (header_buffer[1] != 'T' as u8) ||
        (header_buffer[2] != 'h' as u8) || (header_buffer[3] != 'd' as u8) {
        return Err(format!("The file {} doesn't start by the correct header", filename));
    }

    Ok(())
}

fn read_header_size(mut file: &mut std::fs::File, filename: &str) -> Result<(), String>
{
    match file.read_u32::<BigEndian>() {
        Ok(6) => Ok(()),
        Ok(x) => Err(format!("Invalid header size found in file {}. Expecting 6, got {}", filename, x)),
        Err(e) => Err(format!("Failed to read header size out of file {}: {}", filename, e.description())),
    }
}

#[derive(PartialEq)]
enum MidiType
{
    SingleTrack = 0,
    MultipleTrack = 1,
    MultipleSong = 2, // i.e. a series of type 0
}

fn read_midi_type(mut file: &mut std::fs::File, filename: &str) -> Result<MidiType, String>
{
    match file.read_u16::<BigEndian>() {
        Ok(0) => Ok(MidiType::SingleTrack),
        Ok(1) => Ok(MidiType::MultipleTrack),
        Ok(2) => Ok(MidiType::MultipleSong),
        Ok(e) => Err(format!("Invalid midi file type for file {}. Expecting either 0 (single track), 1 (multiple track) or 2 (multiple song), got {}", filename, e)),
        Err(e) => Err(format!("Failed to read the midi type of {}: {}", filename, e.description())),
    }
}

fn read_nb_tracks(mut file: &mut std::fs::File, filename: &str) -> Result<u16, String>
{
    match file.read_u16::<BigEndian>() {
        Ok(e) => Ok(e),
        Err(e) => Err(format!("Failed to read the number of tracks of file {}: {}", filename, e.description())),
    }
}

enum TempoStyle {
    MetricalTiming,
    Timecode,
}

fn get_tickdiv(mut file: &mut std::fs::File, filename: &str) -> Result<(u16, TempoStyle), String>
{
    // http://midi.mathewvp.com/aboutMidi.htm

    // The last two bytes indicate how many Pulses (i.e. clocks) Per Quarter Note
    // (abbreviated as PPQN) resolution the time-stamps are based upon,
    // Division. For example, if your sequencer has 96 ppqn, this field would be
    // (in hex): 00 60 Alternately, if the first byte of Division is negative,
    // then this represents the division of a second that the time-stamps are
    // based upon. The first byte will be -24, -25, -29, or -30, corresponding to
    // the 4 SMPTE standards representing frames per second. The second byte (a
    // positive number) is the resolution within a frame (ie, subframe). Typical
    // values may be 4 (MIDI Time Code), 8, 10, 80 (SMPTE bit resolution), or 100.
    // You can specify millisecond-based timing by the data bytes of -25 and 40
    // subframes.
    let mut bytes : [u8; 2] = [0; 2];

    if let Err(e) = file.read_exact(&mut bytes) {
        return Err(format!("Failed to read data from file {}: {}", filename, e.description()));
    }

    if (bytes[0] as i8) >= 0
        {
            let tickdiv : u16 = ((bytes[0] as u16 ) << 8) | (bytes[1] as u16);
            return Ok((tickdiv, TempoStyle::MetricalTiming));
        }
        else
        {
            let frames_per_sec : u8 = (-(bytes[0] as i8)) as u8;
            match frames_per_sec {
                24 | 25 | 29 | 30 => {
                    let resolution : u8 = bytes[1];
                    let res : u16 = resolution as u16 * frames_per_sec as u16;
                    Ok((res, TempoStyle::Timecode))
                },
                _ => Err("Error: midi file contain an invalid number of frames per second".to_owned())
            }
        }

}

pub fn get_midi_events(filename: &str) -> Result<Vec<MidiEvent>, String>
{
    let mut file = match std::fs::File::open(filename) {
        Err(e) => {
            return Err(format!("Failed to open file {}: {}", filename, e.description()));
        }
        Ok(f) => f,
    };

    // http://www.ccarh.org/courses/253/handout/smf/
    //
    //    header_chunk = "MThd" + <header_length> + <format> + <n> + <division>
    //
    // "MThd" 4 bytes
    //     the literal string MThd, or in hexadecimal notation: 0x4d546864. These
    //     four characters at the start of the MIDI file indicate that this is a
    //     MIDI file.
    //
    // <header_length> 4 bytes
    //     length of the header chunk (always 6 bytes long--the size of the next
    //     three fields which are considered the header chunk).
    //
    // <format> 2 bytes
    //     0 = single track file format
    //     1 = multiple track file format
    //     2 = multiple song file format (i.e., a series of type 0 files)
    //
    // <n> 2 bytes
    //     number of track chunks that follow the header chunk
    //
    // <division> 2 bytes
    //     unit of time for delta timing. If the value is positive, then it
    //     represents the units per beat. For example, +96 would mean 96 ticks
    //     per beat. If the value is negative, delta times are in SMPTE
    //     compatible units.


    read_magic_number(&mut file, filename)?;
    read_header_size(&mut file, filename)?;
    let midi_type = read_midi_type(&mut file, filename)?;
    if midi_type == MidiType::MultipleSong {
        return Err("this program doesn't handle multiple song midi files - yet -".to_owned())
    }

    let nb_tracks = read_nb_tracks(&mut file, filename)?;
    if (midi_type == MidiType::SingleTrack) && (nb_tracks != 1) {
        return Err(format!("Midi file {} is supposed to be a single track one but it says it contains {} tracks", filename, nb_tracks));
    }

    let (tickdiv, _ /*timing_type*/) = get_tickdiv(&mut file, filename)?;
    if tickdiv == 0 {
        return Err("Error: a quarter note is made of 0 pulses (which is impossible) according to the midi data".to_owned());
    }

    return Ok(Vec::new());
}

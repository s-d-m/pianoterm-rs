extern crate byteorder;
use midi_reader::byteorder::{ReadBytesExt, BigEndian};

use std::io::prelude::*;
use std::error::Error;
use std::io::SeekFrom;
use std;

fn is_header_correct(actual_buffer: [u8;4], expected: [u8;4]) -> bool
{
    for i in 0..4 {
        if actual_buffer[i] != expected[i] {
            return false;
        }
    }
    true
}

fn read_magic_number(mut file: &mut std::fs::File, filename: &str) -> Result<(), String>
{
    let mut header_buffer: [u8; 4] = [0; 4];
    if let Err(e) = file.read_exact(&mut header_buffer) {
        return Err(format!("Failed to read the header out of file {}: {}", filename, e.description()));
    }

    let midi_header : [u8;4] = ['M' as u8, 'T' as u8, 'h' as u8, 'd' as u8];
    if !is_header_correct(header_buffer, midi_header) {
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

fn read_one_byte(mut file: &mut std::fs::File) -> Result<u8, std::io::Error>
{
    let mut buffer: [u8; 1] = [0; 1];
    file.read_exact(&mut buffer)?;
    return Ok(buffer[0]);
}

fn get_variable_length_array(mut file: &mut std::fs::File) -> Result<Vec<u8>, String>
{
    let mut res = Vec::<u8>::new();

    loop {
        match read_one_byte(&mut file) {
            Err(e) => return Err(format!("Failed to read byte: {}", e.description())),
            Ok(v) =>
                {
                    res.push(v);
                    if (v & 0x80) == 0 {
                        // the continuation bit is off, this is the last byte
                        break;
                    }
                }
        }
    }

    Ok(res)
}

fn get_variable_length_value(vec: &[u8]) -> Result<u64, String>
{
    if vec.len() > 8 {
        return Err("buffer is too big to extract a variable length value from".to_owned());
    }

    let mut res : u64 = 0;
    for i in 0..vec.len() {
        res = (res << 7) | (vec[i] & 0x7F) as u64;
    }

    Ok(res)
}

// reads a variable length value. BUT it must be four bytes maximum.
// otherwise it is not valid.
fn get_relative_time(mut file: &mut std::fs::File) -> Result<u32, String>
{
    // recreate the right value by removing the continuation bits
    let buffer = get_variable_length_array(&mut file)?;

    if buffer.len() > 4 {
        return Err(format!("Invalid relative timing found.\nMaximum size allowed is 4 bytes. Bytes used: {}", buffer.len()));
    }

    let res = get_variable_length_value(&buffer)?;
    let res = res as u32;
    Ok(res)
}


pub struct MidiEvent {
   pub time: u64,
   pub data: Vec<u8>,
}

impl MidiEvent {

    pub fn is_key_pressed(&self) -> bool
    {
        return (self.data.len() == 3) && ((self.data[0] & 0xF0) == 0x90) && (self.data[2] != 0x00);
    }

    pub fn is_key_released(&self) -> bool
    {
        return (self.data.len() == 3) &&
            ((self.data[0] & 0xF0 == 0x80) || (((self.data[0] & 0xF0) == 0x90) && (self.data[2] == 0x00)));
    }

    pub fn get_pitch(&self) -> Option<u8>
    {
        match self.data.len() {
            x if x >= 2 => Some(self.data[1]),
            _ => None
        }
    }
}

// return the next byte of the file without extracting it.
fn peek_byte(mut file: &mut std::fs::File) -> Result<u8, String>
{
    match read_one_byte(&mut file) {
        Err(e) => Err(format!("Failed to read one byte: {}", e.description())),
        Ok(x) => {
            match file.seek(SeekFrom::Current(-1)) {
                Err(e) => Err(format!("Failed to change the cursor position within the file {}", e.description())),
                Ok(_) => Ok(x),
            }
        },
    }
}

fn get_event(mut file: &mut std::fs::File, last_status_byte: u8) -> Result<MidiEvent, String>
{
    // get_relative_time returns MIDI tics. These are the number of tics that occured since the former event
    // if the song uses the metrical timing tempo_style, or since the beginning of the song if it used the timecode
    // tempo_style. These are *NOT* in a dimension of seconds. Therefore assigning them to res.time which is of type
    // nanoseconds is wrong here. instead, there should which denote a midi tics that should be used here.
    // In order not to create too many types, and avoid some memory and copies overhead, the .time field is used here
    // to stop the midi tics.
    // There will be a step later on that will transfer these midi tics (wrongly stored as nanoseconds here), into
    // the real naonseconds value, based on the tempo style. See function set_real_timings.

    let time_in_ns = get_relative_time(&mut file)? as u64;
    let mut data : Vec<u8> = Vec::new();

    // we need to look at the next byte in the file (there must be at least one).
    // If the most significant bit is cleared then the event byte is last_status_byte, and the byte
    // we just read from the file should be kept therefore further consumption.
    // otherwise, the byte we looked at _is_ the event byte, and must be taken out of the file.
    let event_type = match peek_byte(&mut file)? {
        x if (x & 0x80) == 0 => last_status_byte,
        x => {
            match file.seek(SeekFrom::Current(1)) { // consume the byte in the file
                Err(e) => return Err(format!("Failed to change the cursor position within the file {}", e.description())),
                Ok(_) => x,
            }
        }
    };

    data.push(event_type);
    if event_type == 0xFF {
        // This is a META event
        match read_one_byte(&mut file) /* type of META event  */ {
            Err(e) => return Err(format!("Error while reading META midi event: {}", e.description())),
            Ok(x) => data.push(x),
        };
        // for the rest, a META event is just like a sysex one.
    }

    if (event_type == 0xFF) || (event_type == 0xF0) || (event_type == 0xF7) {
        // this is a sysex event, or the end of a META event.
        let length_array = get_variable_length_array(&mut file)?;
        let length = get_variable_length_value(&length_array)?;

        // Append the length array at the end of res.data
        for byte in length_array {
            data.push(byte);
        }

        // the data
        for _ in 0..length {
            match read_one_byte(&mut file) {
                Err(e) => return Err(format!("Error while parsing sysex or end of meta midi type: {}", e.description())),
                Ok(byte) => data.push(byte),
            }
        }

        return Ok(MidiEvent{ time: time_in_ns, data: data });
    }

    if ((event_type & 0xF0) >= 0x80) && ((event_type & 0xF0) != 0xF0) {
        if    ((event_type & 0xF0) == 0xC0) /* Program Change Event */
           || ((event_type & 0xF0) == 0xD0) /* or Channel Aftertouch Event */ {
            // one more byte
            match read_one_byte(&mut file) {
                Err(e) => return Err(format!("Error while parsing a program after change event or a channel aftertouch event: {}", e.description())),
                Ok(byte) => data.push(byte),
            };
        } else {
            // this is a MIDI channel event (more two bytes)
            for _ in 0..2 {
                match read_one_byte(&mut file) {
                    Err(e) => return Err(format!("Error while parsing Midi channel event: {}", e.description())),
                    Ok(byte) => data.push(byte),
                };
            }
        }

        return Ok(MidiEvent{ time: time_in_ns, data: data});
    }

    return Err("Invalid Midi file. Unknown Midi event".to_owned());
}


// read the midi events from the track and pushes them at the end of res
//
// MIDI format 1 (multiple track) can't have tempo event after the first track.
// call with the last to true when reading track 2+ from a format 1 to ensure
// validity check.
fn get_track_events(mut res: &mut Vec<MidiEvent>, mut file: &mut std::fs::File, fail_on_tempo_event: bool) -> Result<(), String>
{
    // http://www.ccarh.org/courses/253/handout/smf/
    //
    // A track chunk consists of a literal identifier string, a length indicator
    // specifying the size of the track, and actual event data making up the track.
    //
    //    track_chunk = "MTrk" + <length> + <track_event> [+ <track_event> ...]
    //
    // "MTrk" 4 bytes
    //     the literal string MTrk. This marks the beginning of a track.
    // <length> 4 bytes
    //     the number of bytes in the track chunk following this number.
    // <track_event>
    //     a sequenced track event.

    let mut track_buffer: [u8; 4] = [0; 4];
    if let Err(e) = file.read_exact(&mut track_buffer) {
        return Err(format!("Failed to read the header out of file: {}", e.description()));
    }


    let track_header : [u8;4] = [ 'M' as u8, 'T' as u8, 'r' as u8, 'k' as u8];
    if !is_header_correct(track_buffer, track_header) {
       return Err(format!("Invalid midi file. Couldn't read the track header"));
    }

    let track_length : u32 = match file.read_u32::<BigEndian>() {
        Err(e) => return Err(format!("Error while reading file: {}", e.description())),
        Ok(v) => v,
    };

    let track_start = match file.seek(SeekFrom::Current(0)) {
        Err(e) => return Err(format!("Failed to get the position into the file for the beginning of a track: {}", e.description())),
        Ok(v) => v,
    };

    let mut last_status_byte : u8 = 0x00;
    let mut this_time_in_ns : u64 = 0; // unit is nanoseconds

    loop {
        let event = get_event(&mut file, last_status_byte)?;

        // the time of an event was given related to the former event.
        // this is about making it relative to the beginning of the song
        let event = MidiEvent{ time: event.time + this_time_in_ns, data: event.data};
        this_time_in_ns = event.time;

        last_status_byte = event.data[0];

        if (event.data[0] == 0xFF) && (event.data[1] == 0x51) // This is a tempo event
            && fail_on_tempo_event {
            return Err("Error: a tempo event found at a forbidden place".to_owned());
        }

        let end_of_track_found  = (event.data[0] == 0xFF) && (event.data[1] == 0x2F);

        res.push(event);

        if end_of_track_found {
            break;
        }
    }

    let track_end = match file.seek(SeekFrom::Current(0)) {
        Err(e) => return Err(format!("Failed to get the position into the file for the end of a track: {}", e.description())),
        Ok(v) => v,
    };

    if track_end - track_start != track_length as u64 {
        return Err("Error: invalid track length detected".to_owned());
    }

    Ok(())
}

fn set_real_timings(mut events : &mut [MidiEvent], tickdiv: u16, timing_style : TempoStyle) -> Result<(), String>
{
    // pre condition, events must be sorted!
    for i in 1..events.len() {
        if events[i].time < events[ i - 1].time {
            panic!("events must be sorted by time!".to_owned());
        }
    }

    match timing_style {
        TempoStyle::Timecode => {
            for event in events {
                event.time = event.time * tickdiv as u64 * 1_000_000;
            }
        },
        TempoStyle::MetricalTiming => {
            let mut ref_ticks : u64 = 0;
            let mut ref_time_in_ns : u64 = 0;

            // default tempo is 120 beats per minutes
            // 1 minute -> 60 000 000 microseconds
            // 60000000 / 120 -> 500 000 microseconds per quarter note
            let mut us_per_quarter_note : u64 = 500_000;
            for event in events.iter_mut() {
                let last_ticks = event.time;
                let delta_ticks = last_ticks - ref_ticks;
                event.time = ref_time_in_ns + ((delta_ticks * us_per_quarter_note * 1_000) / tickdiv as u64);

                if (event.data[0] == 0xFF) && (event.data[1] == 0x51) {
                    // this is a tempo event
                    if event.data.len() != 6 {
                        return Err("tempo has an invalid size".to_owned());
                    }

                    ref_ticks = last_ticks;
                    ref_time_in_ns = event.time;

                    us_per_quarter_note = ((event.data[3] as u64) << 16) | ((event.data[4] as u64) << 8) | (event.data[5] as u64);
                }
            }
        },
    }

    return Ok(());
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

    let (tickdiv, timing_type) = get_tickdiv(&mut file, filename)?;
    if tickdiv == 0 {
        return Err("Error: a quarter note is made of 0 pulses (which is impossible) according to the midi data".to_owned());
    }

    let mut events : Vec<MidiEvent> = Vec::new();

    for i in 0..nb_tracks {
        get_track_events(&mut events, &mut file, (midi_type == MidiType::MultipleTrack) && (i != 0))?;
    }

    // by now the whole file should have been read
    let metadata = match file.metadata() {
        Err(e) => return Err(format!("Failed to read the file size: {}", e.description())),
        Ok(v) => v,
    };

    let file_size = metadata.len();

    let nb_read_bytes = match file.seek(SeekFrom::Current(0)) {
        Err(e) => return Err(format!("Failed to get the position into the file for the end of a track: {}", e.description())),
        Ok(v) => v,
    };

    if nb_read_bytes != file_size {
        return Err("file contains extra bytes after end of MIDI data".to_owned());
    }

    events.sort_by(|a, b| match (a, b) {
        (a, b) if a.time < b.time => std::cmp::Ordering::Less,
        (a, b) if a.time > b.time => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });

    set_real_timings(&mut events, tickdiv, timing_type)?;

    // keep only the midi events
    let mut res : Vec<MidiEvent> = Vec::new();

    for event in events {
        if (event.data[0] & 0xF0) != 0xF0 {
            res.push(event);
        }
    }
    return Ok(res);
}

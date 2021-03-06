use std;
use keyboard_events_extractor::{KeyEvent, KeyData};
use midi_reader::MidiEvent;

pub const LA_0: u8 = 21;
pub const LA_DIESE_0: u8 = 22;
pub const SI_0: u8 = 23;
pub const DO_1: u8 = 24;
pub const DO_8: u8 = 108;

pub type MidiMessage = Vec<u8>;

pub struct MusicEvent {
    pub time_in_ns: u64,
    pub midi_messages: Vec<MidiMessage>,
    pub key_events: Vec<KeyData>,
}

pub type Song = Vec<MusicEvent>;


fn is_key_release_event(data: &[u8]) -> bool {
    (data.len() == 3) &&
    (((data[0] & 0xF0) == 0x80) || (((data[0] & 0xF0) == 0x90) && (data[2] == 0x90)))
}

fn is_key_down_event(data: &[u8]) -> bool {
    (data.len() == 3) && ((data[0] & 0xF0) == 0x90) && (data[2] != 0x00)
}

fn get_variable_data_length(message_stream: &[u8]) -> usize {
    let max_res = message_stream.len();

    let mut array_size : usize = 0;
    let mut nb_read = 0;

    for c in message_stream.iter() {
        nb_read += 1;
        array_size = (array_size << 7) + (c & 0x7F) as usize;
        if (c & 0x80) == 0  {
            return std::cmp::min(max_res, array_size + nb_read);
        }
    }

    // at this point input is invalid
    return max_res;
}


fn get_next_event_size(message_stream: &[u8]) -> usize {
    let len = message_stream.len();

    if len < 2 {
        // minimal possible length (midi channel event 0xC0 and 0xD0
        return len;
    }

    let mut res = 1;  // the first byte which the event type (channel, meta, sysex)
    let ev_type = message_stream[0];
    if ev_type == 0xFF {
        // META event has one byte more than sysex
        res += 1;
    }

    if (ev_type == 0xFF) || (ev_type == 0xF0) || (ev_type == 0xF7) {
        // end of META or sysex
        res += get_variable_data_length(&message_stream[res..]);

        // sanity check: in case of wrong input, simply discard data
        return std::cmp::min(res, len);
    }

    if ((ev_type & 0xF0) >= 0x80) && ((ev_type & 0xF0) != 0xF0) {
        if ((ev_type & 0xF0) == 0xC0)
	    || ((ev_type & 0xF0) == 0xD0)  /* Program Change Event */
        { /* or Channel Aftertouch Event */
             // one more byte
             res += 1;
        } else {
            // this is a MIDI channel event (more two bytes)
            res += 2;
        }

        return std::cmp::min(res, len);
    }

    // at this point there is an error, discard data
    return len;
}

pub fn midi_to_music_events(message_stream: &[u8]) -> MusicEvent {

    let mut res = MusicEvent{ midi_messages: Vec::<MidiMessage>::new(),
                              time_in_ns: 0,
                              key_events: Vec::<KeyData>::new() };

    let size = message_stream.len();
    let mut nb_read = 0;

    while nb_read < size {
        let this_event_size = get_next_event_size(&message_stream[nb_read .. ]);
        if this_event_size == 0 {
            // this is an error. return what we have found so far to avoid
            // an infinite loop. This can happen with variable length array.
            // If the computation of the size overflow and falls to 0, then
            // ...
            return res;
        }

        if this_event_size == 3 { // can it be a midi key press or key release event?
            let tmp = &message_stream[nb_read .. nb_read + 3];

            let pitch = tmp[1];
            if (pitch >= LA_0) && (pitch <= DO_8) {

                if is_key_release_event(&tmp) {
	            res.key_events.push(KeyData::Released(pitch));
                }
                else if is_key_down_event(&tmp) {
	            res.key_events.push(KeyData::Pressed(pitch));
                }

                res.midi_messages.push(tmp.to_vec());
            }
        }

        nb_read += this_event_size;
    }

    return res;
}

// in case there is a release pitch and a play pitch at the same time
// in the midi part, make sure the release happens *before* the play.
// the release must be from a previous play as otherwise it would mean
// that the a key is pressed and immediately released, so not played
// at all (which is wrong)
fn fix_midi_order(song: &mut Song) {
    for music_event in song.iter_mut() {

        let mut pos_to_switch: Vec<(usize, usize)> = Vec::new();
        for (cur_pos, message) in music_event.midi_messages.iter().enumerate() {
            let midi_ev = &message;
            if is_key_release_event(midi_ev) {
                let pitch = midi_ev[1];
                match music_event.midi_messages[cur_pos + 1..]
                          .iter()
                          .position(|&ref x| is_key_down_event(&x) && (x[1] == pitch)) {
                    Some(down_dist) => pos_to_switch.push((cur_pos, cur_pos + down_dist)),
                    _ => (),
                }
            }
        }

        for (a, b) in pos_to_switch {
            music_event.midi_messages.swap(a, b);
        }
    }
}


pub fn group_events_by_time(midi_events: &Vec<MidiEvent>,
                            keyboard_events: &Vec<KeyEvent>)
                            -> Result<Song, String> {
    let mut res: Song = Vec::new();

    // totally suboptimal implementation
    for elt in midi_events {
        let time = elt.time;
        match res.iter().position(|ref x| x.time_in_ns == time) {
            None => {
                res.push(MusicEvent {
                             time_in_ns: time,
                             midi_messages: vec![elt.data.clone()],
                             key_events: vec![],
                         })
            }
            Some(pos) => res[pos].midi_messages.push(elt.data.clone()),
        }
    }

    for k in keyboard_events {
        let ev_time = k.time_in_ns;
        match res.iter().position(|ref x| x.time_in_ns == ev_time) {
            None => {
                res.push(MusicEvent {
                             time_in_ns: ev_time,
                             midi_messages: vec![],
                             key_events: vec![k.data],
                         })
            }
            Some(pos) => res[pos].key_events.push(k.data),
        }
    }

    res.sort_by(|a, b| match (a, b) {
                    (a, b) if a.time_in_ns < b.time_in_ns => std::cmp::Ordering::Less,
                    (a, b) if a.time_in_ns == b.time_in_ns => std::cmp::Ordering::Equal,
                    (a, b) if a.time_in_ns > b.time_in_ns => std::cmp::Ordering::Greater,
                    _ => panic!("comparison is not total!"),
                });

    // sanity check: all elements in res must hold at least one event
    if let Some(_) = res.iter()
           .find(|x| x.midi_messages.is_empty() && x.key_events.is_empty()) {
        return Err("Error: a music event does not contain any midi or key event".to_owned());
    }

    // sanity check: worst case res has as many elts as midi_events + key_events
    // (each event occuring at a different time)
    let nb_input_events = midi_events.len() + keyboard_events.len();
    if res.len() > nb_input_events {
        return Err("Error while grouping events by time, some events just got automagically created".to_owned());
    }

    // sanity check: count the total number of midi and key events in res. It must
    // match the number of parameters given in the parameters
    let nb_events = res.iter()
        .map(|x| x.midi_messages.len() + x.key_events.len())
        .sum::<usize>();

    if nb_events > nb_input_events {
        return Err("Error while grouping events by time, some events magically appeared".to_owned());
    }

    if nb_events < nb_input_events {
        return Err("Error while grouping events by time, some events just disappeared".to_owned());
    }

    // sanity check: for every two different elements in res, they must start at different time
    // since res is sorted by now, only need to check
    for i in 1..res.len() {
        if res[i - 1].time_in_ns == res[i].time_in_ns {
            return Err("Error two different group of events appears at the same time".to_owned());
        }
    }

    // sanity check: there must be as many release events as pressed events
    let count_key_released_events = |key_events: &Vec<KeyData>| {
        key_events
            .iter()
            .filter(|&elt| match *elt {
                        KeyData::Released(_) => true,
                        KeyData::Pressed(_) => false,
                    })
            .count()
    };

    let count_key_pressed_events = |key_events: &Vec<KeyData>| {
        key_events
            .iter()
            .filter(|&elt| match *elt {
                        KeyData::Released(_) => false,
                        KeyData::Pressed(_) => true,
                    })
            .count()
    };

    let nb_released = res.iter()
        .map(|ref x| count_key_released_events(&x.key_events))
        .sum::<usize>();
    let nb_pressed = res.iter()
        .map(|ref x| count_key_pressed_events(&x.key_events))
        .sum::<usize>();

    if nb_pressed != nb_released {
        return Err("Error: mismatch between key pressed and key release".to_owned());
    }

    // sanity check: a key release and a key pressed event with the same pitch
    // can't appear at the same time
    for elt in res.iter() {
        for k in elt.key_events.iter() {
            if let KeyData::Released(pitch) = *k {
                if elt.key_events
                       .iter()
                       .find(|&x| match *x {
                                 KeyData::Pressed(p) if p == pitch => true,
                                 _ => false,
                             })
                       .is_some() {
                    return Err("Error: a key press happens at the same time as a key release".to_owned());
                }
            }
        }

    }

    fix_midi_order(&mut res);

    Ok(res)
}

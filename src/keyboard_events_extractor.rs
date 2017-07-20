use std::cmp;
use midi_reader;

pub enum KeyData
{
    Pressed(u8), // u8 is the pitch
    Released(u8),
}

pub struct KeyEvent
{
    pub data: KeyData,
    pub time_in_ns: u64,
}

fn separate_released_pressed_keys(mut keyboard_events: &mut Vec<KeyEvent>) -> Result<(), String>
{
    // precond the events MUST be sorted by time. this function only works on that case
    for i in 1..keyboard_events.len() {
        if keyboard_events[i].time_in_ns < keyboard_events[ i - 1].time_in_ns {
            panic!("keyboard events must be sorted by time!".to_owned());
        }
    }

    // for each pressed event, look if there is another pressed event that happens
    // at the exact same time as its associated release event. If so, shorten the
    // duration of the former pressed event (i.e advance the time the release
    // event occurs).

    // suboptimal implementation in the case the key_events are sorted
    let mut pos_shortening_time : Vec<(usize, u64)> = Vec::new();
    for k in keyboard_events.iter() {
        if let KeyData::Pressed(pitch) = k.data {
            let earliest_time = k.time_in_ns;

            // is there a release happening at the same time?
            let find_released_pos_fn = |x: &KeyEvent| match (x.time_in_ns, &x.data) {
                (time, &KeyData::Released(this_pitch)) if (time == earliest_time) && (this_pitch == pitch) => true,
                (_, _) => false,
            };

            if let Some(released_elt_index) = keyboard_events.iter().position(find_released_pos_fn) {
                // there do is a release key happening at the same time.
                // Let's find the pressed key responsible for it
                let find_prev_pressed_fn = |x: &&KeyEvent| match (x.time_in_ns, &x.data) {
                    (time, &KeyData::Pressed(this_pitch)) if (time < earliest_time) && (this_pitch == pitch) => true,
                    (_, _) => false,
                };

                match keyboard_events.iter().rev().find(find_prev_pressed_fn) {
                    Some(pressed_key_pos) => {
                        // compute the shortening time
                        let duration = earliest_time - pressed_key_pos.time_in_ns;
                        let max_shortening_time_in_ns : u64 = 75_000_000; // 75 ms

                        // shorten the duration by one fourth of its time, in the worst case
                        let shortening_time = cmp::min(max_shortening_time_in_ns, duration / 4);

                        pos_shortening_time.push((released_elt_index, shortening_time));
                    },
                    None => return Err("error, a there is release event coming from nowhere (failed to find the associated pressed event)".to_owned()),
                }
            }

        }
    }

    for &(pos, shortening_time) in pos_shortening_time.iter() {
        keyboard_events[pos].time_in_ns -= shortening_time;
    }

    return Ok(());
}

pub fn get_key_events(midi_events: Vec<midi_reader::MidiEvent>) -> Result<Vec<KeyEvent>, String>
{
    // pre condition, events must be sorted!
    for i in 1..midi_events.len() {
        if midi_events[i].time < midi_events[ i - 1].time {
            panic!("midi events must be sorted by time!".to_owned());
        }
    }

    let mut res : Vec<KeyEvent> = Vec::new();

    for ev in midi_events {
        if ev.is_key_released() {
            res.push(KeyEvent{data: KeyData::Released(ev.get_pitch().unwrap()),
                              time_in_ns: ev.time})
        }

        if ev.is_key_pressed() {
            res.push(KeyEvent{data: KeyData::Pressed(ev.get_pitch().unwrap()),
                              time_in_ns: ev.time})
        }

        // sanity check

        if ev.is_key_pressed() && ev.is_key_released() {
            panic!("How come a key is said to be both pressed and released at the same time?");
        }
    }

    // pre condition, res must be sorted!
    for i in 1..res.len() {
        if res[i].time_in_ns < res[ i - 1].time_in_ns {
            panic!("keyboard events must be sorted by time!".to_owned());
        }
    }

    separate_released_pressed_keys(&mut res)?;

    return Ok(res);
}

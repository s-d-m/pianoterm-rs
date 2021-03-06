extern crate midir;
extern crate rustbox;

use std::error::Error;
use std;
use utils;
use self::rustbox::{RustBox, Event, Key};
use keyboard_events_extractor::KeyData;
use std::sync::atomic::Ordering;
use signal_handler::{EXIT_REQUESTED_BY_SIGNAL, PAUSE_REQUESTED_BY_SIGNAL, CONTINUE_REQUESTED_BY_SIGNAL};

fn draw_piano_key(ui: &RustBox, x: usize, y: usize, width: usize, height: usize, color: u16)
{
    for i in x .. x + width {
        for j in y .. y + height {
            unsafe {
                ui.change_cell(i, j, 0x2588, color, rustbox::Color::Default.as_16color());
            }
         }
    }
}

fn draw_separating_line(ui: &RustBox, x: usize, y: usize, height: usize, bg_color: u16)
{
    for j in y .. y + height {
        unsafe {
            ui.change_cell(x, j, 0x2502, rustbox::Color::Black.as_16color(), bg_color);
        }
    }
}

#[derive(Clone, Copy)]
struct OctaveColor
{
    do_color :u8,
    re_color :u8,
    mi_color :u8,
    fa_color :u8,
    sol_color :u8,
    la_color :u8,
    si_color :u8,

    do_diese_color :u8,
    re_diese_color :u8,
    fa_diese_color :u8,
    sol_diese_color :u8,
    la_diese_color :u8,
}

impl OctaveColor
{
    pub fn new() -> Self
    {
        OctaveColor {
            do_color:  rustbox::Color::White.as_16color() as u8,
            do_diese_color:  rustbox::Color::Black.as_16color() as u8,
            re_color:  rustbox::Color::White.as_16color() as u8,
            re_diese_color:  rustbox::Color::Black.as_16color() as u8,
            mi_color:  rustbox::Color::White.as_16color() as u8,
            fa_color:  rustbox::Color::White.as_16color() as u8,
            fa_diese_color:  rustbox::Color::Black.as_16color() as u8,
            sol_color: rustbox::Color::White.as_16color() as u8,
            sol_diese_color: rustbox::Color::Black.as_16color() as u8,
            la_color:  rustbox::Color::White.as_16color() as u8,
            la_diese_color:  rustbox::Color::Black.as_16color() as u8,
            si_color:  rustbox::Color::White.as_16color() as u8,
        }
    }
}

struct KeysColor
{
    la_0_color: u8,
    la_diese_0_color: u8,
    si_0_color: u8,
    octaves: [OctaveColor; 7],
    do_8_color: u8,
}

impl KeysColor
{
    pub fn new() -> Self
    {
        KeysColor {
            la_0_color:  rustbox::Color::White.as_16color() as u8,
            la_diese_0_color:  rustbox::Color::Black.as_16color() as u8,
            si_0_color:  rustbox::Color::White.as_16color() as u8,
            octaves: [OctaveColor::new(); 7],
            do_8_color:  rustbox::Color::White.as_16color() as u8,
        }
    }

    fn set_color_(&mut self, pitch: u8, normal_key_color: rustbox::Color, diese_key_color: rustbox::Color) {
        match pitch {
            utils::LA_0 => self.la_0_color = normal_key_color.as_16color() as u8,
            utils::LA_DIESE_0 => self.la_diese_0_color = diese_key_color.as_16color() as u8,
            utils::SI_0 => self.si_0_color = normal_key_color.as_16color() as u8,
            p if (p >= utils::DO_1) && (p < utils::DO_8) => {
                let octave_number = ((p - utils::DO_1) / 12) as usize;
                let key_pos = (p - utils::DO_1) % 12;
                match key_pos {
                    0 => self.octaves[octave_number].do_color = normal_key_color.as_16color() as u8,
                    1 => self.octaves[octave_number].do_diese_color = diese_key_color.as_16color() as u8,
                    2 => self.octaves[octave_number].re_color = normal_key_color.as_16color() as u8,
                    3 => self.octaves[octave_number].re_diese_color = diese_key_color.as_16color() as u8,
                    4 => self.octaves[octave_number].mi_color = normal_key_color.as_16color() as u8,
                    5 => self.octaves[octave_number].fa_color = normal_key_color.as_16color() as u8,
                    6 => self.octaves[octave_number].fa_diese_color = diese_key_color.as_16color() as u8,
                    7 => self.octaves[octave_number].sol_color = normal_key_color.as_16color() as u8,
                    8 => self.octaves[octave_number].sol_diese_color = diese_key_color.as_16color() as u8,
                    9 => self.octaves[octave_number].la_color = normal_key_color.as_16color() as u8,
                    10 => self.octaves[octave_number].la_diese_color = diese_key_color.as_16color() as u8,
                    11 => self.octaves[octave_number].si_color = normal_key_color.as_16color() as u8,
                    e => panic!("invalid pitch for keyboard set: values should be between {} (la_0) and {} (do_8), but got {}", utils::LA_0, utils::DO_8, e),
                }
            },
            x if x == utils::DO_8 => self.do_8_color = normal_key_color.as_16color() as u8,
            e => panic!("invalid pitch for keyboard set: values should be between {} (la_0) and {} (do_8), but got {}", utils::LA_0, utils::DO_8, e),

        }
    }

    pub fn reset_color(&mut self, pitch: u8) {
        self.set_color_(pitch, rustbox::Color::White, rustbox::Color::Black);
    }

    pub fn set_color(&mut self, pitch: u8) {
        self.set_color_(pitch, rustbox::Color::Blue, rustbox::Color::Cyan);
    }

}

fn draw_octave(ui: &RustBox, x: usize, y: usize, notes_color: &OctaveColor)
{
  draw_piano_key(ui, x,     y, 3, 8, notes_color.do_color as u16);  // do
  draw_piano_key(ui, x + 3, y, 4, 8, notes_color.re_color as u16);  // re
  draw_piano_key(ui, x + 7, y, 3, 8, notes_color.mi_color as u16);  // mi

  draw_piano_key(ui, x + 10, y, 4, 8, notes_color.fa_color as u16); // fa
  draw_piano_key(ui, x + 14, y, 4, 8, notes_color.sol_color as u16); // sol
  draw_piano_key(ui, x + 18, y, 3, 8, notes_color.la_color as u16); // la
  draw_piano_key(ui, x + 21, y, 4, 8, notes_color.si_color as u16); // si

  draw_piano_key(ui, x + 2, y, 2, 5, notes_color.do_diese_color as u16);  // do#
  draw_piano_key(ui, x + 6, y, 2, 5, notes_color.re_diese_color as u16);  // re#

  draw_separating_line(ui, x + 3, y + 5, 3, notes_color.do_color as u16); // between do and re
  draw_separating_line(ui, x + 6, y + 5, 3, notes_color.re_color as u16); // between re and mi

  draw_piano_key(ui, x + 13, y, 2, 5, notes_color.fa_diese_color as u16); // fa#
  draw_piano_key(ui, x + 17, y, 2, 5, notes_color.sol_diese_color as u16); // sol#
  draw_piano_key(ui, x + 21, y, 2, 5, notes_color.la_diese_color as u16); // la#

  draw_separating_line(ui, x + 14, y + 5, 3, notes_color.fa_color as u16); // between fa and sol
  draw_separating_line(ui, x + 18, y + 5, 3, notes_color.sol_color as u16); // between sol and la
  draw_separating_line(ui, x + 21, y + 5, 3, notes_color.la_color as u16); // between la and si

  draw_separating_line(ui, x + 10, y, 8, notes_color.mi_color as u16); // between mi and fa
}

fn draw_keyboard(ui: &RustBox, keyboard: &KeysColor, pos_x: usize, pos_y: usize)
{
  draw_piano_key(ui, pos_x + 1, pos_y, 3, 8, keyboard.la_0_color as u16); // la 0
  draw_piano_key(ui, pos_x + 4, pos_y, 4, 8, keyboard.si_0_color as u16); // si 0
  draw_piano_key(ui, pos_x + 4, pos_y, 2, 5, keyboard.la_diese_0_color as u16); // la# 0
  draw_separating_line(ui, pos_x + 4, pos_y + 5, 3, keyboard.la_0_color as u16); // between la0 and si0

  for i in 0 .. 7  {
    draw_octave(ui, pos_x + 8 + (25 * i), pos_y, &keyboard.octaves[i]);
  }

  draw_piano_key(ui, pos_x + 8 + (25 * 7), pos_y, 4, 8, keyboard.do_8_color as u16); // do 8

  for i in 0 .. 7  {
    draw_separating_line(ui, pos_x + 8 + (25 * (i + 1)), pos_y, 8, keyboard.octaves[i].si_color as u16); // between octaves
  }

  draw_separating_line(ui, pos_x + 8 + (25 * 0), pos_y, 8, keyboard.si_0_color as u16);

}

fn play_music(midi_out: &mut midir::MidiOutputConnection, event: &[utils::MidiMessage])
{
    for message in event.iter() {
        if let Err(e) = midi_out.send(&message) {
            println!("Error occured while playing some event: {}", e.description());
            if let Some(e) = e.cause() {
                println!("{}", e.description());
            }
        }
    }
}

fn init_ref_pos(width: usize, height: usize) -> (usize, usize) {
    let keyboard_height = 8;
    let keyboard_width = 188;

    let ref_x = if width > keyboard_width { (width - keyboard_width) / 2 } else { 0 };
    let ref_y = if height > keyboard_height { (height - keyboard_height) / 2 } else { 0 };

    (ref_x, ref_y)
}

fn update_keyboard(keyboard: &mut KeysColor, key_events: &[KeyData]) {
    for k_ev in key_events {
        match *k_ev {
            KeyData::Pressed(pitch) => keyboard.set_color(pitch),
            KeyData::Released(pitch) => keyboard.reset_color(pitch),
        }
    }
}

fn update_screen(ui: &RustBox, keyboard: &KeysColor, ref_x: usize, ref_y: usize)
{
    ui.clear();
    draw_keyboard(ui, keyboard, ref_x, ref_y);
    ui.print(ref_x, ref_y + 10, rustbox::RB_BOLD, rustbox::Color::Magenta, rustbox::Color::Default, "press <CTRL + q> to quit");
    ui.print(ref_x, ref_y + 11, rustbox::RB_BOLD, rustbox::Color::Magenta, rustbox::Color::Default, "press <space> to pause/unpause");
    ui.present();
}

pub fn play(song: utils::Song, midi_output_port: u32) {
    let mut exit_requested = false;

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
    let ui = RustBox::init(Default::default());
    if let Err(e) = ui {
        println!("Failed to initialise the user interface (rustbox): {}", e.description());
        return ();
    };

    let ui = ui.unwrap();
    let (mut x, mut y) = init_ref_pos(ui.width(), ui.height());

    let mut keyboard = KeysColor::new();
    let nb_events = song.len();
    for i in 0 .. nb_events {

        let current_event = &song[i];
        update_keyboard(&mut keyboard, &current_event.key_events);
        update_screen(&ui, &keyboard, x, y);
        play_music(&mut conn_out, &current_event.midi_messages);

        if i != nb_events - 1 {
            let time_to_wait = song[i + 1].time_in_ns - current_event.time_in_ns;
            let one_billion = 1_000_000_000;
            let time_to_wait = std::time::Duration::new(time_to_wait / one_billion, (time_to_wait % one_billion) as u32);

            let started_time = std::time::Instant::now();
            let mut is_in_pause = false;

            loop {
                if EXIT_REQUESTED_BY_SIGNAL.load(Ordering::Relaxed) {
                    EXIT_REQUESTED_BY_SIGNAL.store(false, Ordering::Relaxed);
                    exit_requested = true;
                }

                if PAUSE_REQUESTED_BY_SIGNAL.load(Ordering::Relaxed) {
                    PAUSE_REQUESTED_BY_SIGNAL.store(false, Ordering::Relaxed);
                    is_in_pause = true;
                }

                if CONTINUE_REQUESTED_BY_SIGNAL.load(Ordering::Relaxed) {
                    CONTINUE_REQUESTED_BY_SIGNAL.store(false, Ordering::Relaxed);
                    is_in_pause = false;
                }

                if exit_requested {
                    return;
                }

                let time_now = std::time::Instant::now();
                let waited_time = time_now - started_time;

                if (!is_in_pause) && (waited_time > time_to_wait) {
                    break;
                }

                let time_to_sleep = {
                    if time_to_wait > waited_time {
                        std::cmp::min(std::time::Duration::from_millis(100), time_to_wait - waited_time)
                    } else {
                        std::time::Duration::from_millis(100)
                    }
                };

                match ui.peek_event(time_to_sleep, false) {
                    Ok(Event::NoEvent)
                    | Ok(Event::MouseEvent(_, _, _)) => (),
                    Ok(Event::KeyEventRaw(_, _, _)) => panic!("Raw event received, whereas the raw parameter to peek_event was set to false!"),
                    Ok(Event::ResizeEvent(w, h)) => {
                        if (w < 0) || (h < 0) {
                            panic!("new window size has negtive components. Can't happen after a successful init!");
                        }
                        let (this_x, this_y) = init_ref_pos(w as usize, h as usize);
                        x = this_x;
                        y = this_y;
                    },
                    Ok(Event::KeyEvent(key)) => {
                        match key {
                            Key::Ctrl('q') => exit_requested = true,
                            Key::Char(' ') => is_in_pause = !is_in_pause,
                            _ => (),
                        }
                    },
                    Err(e) => { println!("Error occured in rustbox: {}", e.description()); return (); },
                };

            }
        }
    }
}

pub fn play_midi_input(midi_input_port: u32, midi_output_port: u32) {

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


    let ui = RustBox::init(Default::default());
    if let Err(e) = ui {
        println!("Failed to initialise the user interface (rustbox): {}", e.description());
        return ();
    };

    let ui = ui.unwrap();
    let (mut x, mut y) = init_ref_pos(ui.width(), ui.height());

    let mut keyboard = KeysColor::new();
    update_screen(&ui, &keyboard, x, y);

    let midi_in = midir::MidiInput::new("Midi input from pianoterm-rs");
    if let Err(e) = midi_in {
        println!("Error occured while initialising the midi input: {}", e.description());
        return ();
    }

    let midi_in = midi_in.unwrap();


    let (tx, rx) = std::sync::mpsc::channel();
    let conn_in = midi_in.connect(midi_input_port, "input midi port from pianoterm-rs", move |_timestamp, message, _| {
        let key_events = utils::midi_to_music_events(message);
        tx.send(key_events).unwrap();
    }, ());

    if let Err(e) = conn_in {
        println!("Failed to open midi input port: {}", e.kind().description());
        return ();
    }

    loop {
        if EXIT_REQUESTED_BY_SIGNAL.load(Ordering::Relaxed) {
            EXIT_REQUESTED_BY_SIGNAL.store(false, Ordering::Relaxed);
            return;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(input_music) => {
                update_keyboard(&mut keyboard, &input_music.key_events);
                play_music(&mut conn_out, &input_music.midi_messages);
                update_screen(&ui, &keyboard, x, y);
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
            Err(e) => {
                println!("Failed to receive input data {}", e.description());
                return;
            },
        };

        match ui.peek_event(std::time::Duration::from_millis(0), false) {
            Ok(Event::NoEvent)
                | Ok(Event::MouseEvent(_, _, _)) => (),
            Ok(Event::KeyEventRaw(_, _, _)) => panic!("Raw event received, whereas the raw parameter to peek_event was set to false!"),
            Ok(Event::ResizeEvent(w, h)) => {
                if (w < 0) || (h < 0) {
                    panic!("new window size has negtive components. Can't happen after a successful init!");
                }
                let (this_x, this_y) = init_ref_pos(w as usize, h as usize);
                x = this_x;
                y = this_y;
                update_screen(&ui, &keyboard, x, y);
            },
            Ok(Event::KeyEvent(key)) => {
                match key {
                    Key::Ctrl('q') => return,
                    _ => (),
                }
            },
            Err(e) => { println!("Error occured in rustbox: {}", e.description()); return (); },
        };
    }
}

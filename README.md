PIANOTERM-RS
============

Pianoterm-rs, as the name suggests, displays a piano in your terminal.
It supports playing midi files, and waiting for events from a midi input port.
Have a look at the following video to get a preview of what it does.

![video demo of pianoterm](./misc/demo.gif?raw=true)

The video has no embedded sound in it, but the real program plays it.

License
------

Todo: choose a license

Build dependencies
----------------

`pianoterm-rs` is implemented in rust and uses cargo to manage the dependencies.
If rust and cargo are not installed yet, follow the instructions decribe at https://www.rustup.rs/

Once rust and cargo are setup, simply compile using

     cargo build --release

How to use
----------

Pianoterm-rs needs a midi sequencer to produce sound. On linux, you can use timidity.
If you decided to use that one, you will need to run it first using:

	timity -iA &

Then, you can list the midi "ports" using

	./target/release/pianoterm-rs --list

This prints something like:

	5 output ports found:
	  0 -> Midi Through 14:0
	  1 -> TiMidity 128:0
	  2 -> TiMidity 128:1
	  3 -> TiMidity 128:2
	  4 -> TiMidity 128:3

Then you can then play a midi file through one of the former sequencer using:

	./target/release/pianoterm-rs --output-port 1 <your_midi_file>

An example midi file is provided in the `misc` folder.

You might also connect a (virtual) keyboard to your computer and use
it in place of the midi file. If such a keyboard is connected it must show up in the listing.
E.g with a [virtual midi keyboard player][vmpk]

[vmpk]: http://sourceforge.net/projects/vmpk/

	./target/release/pianoterm-rs --list

This print something like:

	6 output ports found:
	  0 -> Midi Through 14:0
	  1 -> VMPK Input 129:0
	  2 -> TiMidity 130:0
	  3 -> TiMidity 130:1
	  4 -> TiMidity 130:2
	  5 -> TiMidity 130:3

	2 input ports found:
	  0 -> Midi Through 14:0
	  1 -> VMPK Output 128:0

Now run

	./target/release/pianoterm-rs --input-port 1 --output-port 2

This will use the virtual keyboark (VMPK) as input, and will use `TiMidity 130:0` as the midi sequencer.

Bugs & questions
--------------

Report bugs and questions to da.mota.sam@gmail.com

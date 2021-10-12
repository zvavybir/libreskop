# libreskop
A linux interface for the miniskop.

# Usage
It should just work and output the data on stdout.  The data can be
visualized with GNUPLOT, a simple script doing just that is in `src/`.  Commands to run:

	cargo run > data.plot
	gnuplot src/script.gp

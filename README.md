# phrace

![A bunch of terminal windows showing the tool in action. A Grace window is also shown displaying the same (energy minimization) potential energy data as the terminal that sits above it, for comparison.](screenshot.png)

A terminal viewer for `.xvg` files.

Gromacs tools can output `xvg` files to inspect a range of properties of molecular dynamics trajectories (e.g. [`gmx energy`](https://manual.gromacs.org/current/onlinehelp/gmx-energy.html)).
These data files are commonly viewed with the [Grace](https://plasma-gate.weizmann.ac.il/Grace/) (`xmgrace`) tool.[^grace]

[^grace]: By the way, insanely cool website. Check it out!

## Installation

If you don't have [Rust](https://www.rust-lang.org/) installed on your system, learn how to do it [here](https://www.rust-lang.org/learn/get-started). It is very easy.

To install the binary, run the following command.

```console
cargo install --git https://github.com/koenwestendorp/phrace
```

## Usage

```console
phrace example_data/potential_energy_em.xvg
```

The expected output for this command is:

```
                                       GROMACS Energies
  .

   .

(
k
J  .
/  .=
m   .
o    +:
l     :++:
)        =+++.
             =#+#=+#+
                     #=@++++#+++#+
                                  ++#++#++#=@=#+#++++#+#=
                                                        .+++#++#+++#+#++++#+#++++#+#++++#+#+#+=
                                                                                              .
                                          Time (ps)
Summary:  393 items,  mean ± σ  -1512.5859 ± 54.731205,  min … max  -1565.1497 … -1111.5247
```

## Future work

If I have some time, I want to add more options to change the viewport, add axis labels, ability to plot multiple columns at once.
For now, it fits my needs, and I am putting it out there for others to use in case they find it useful in its current state.

I am interested in adding support for more data formats.
Honestly, it would be pretty cool to add the ability to graph out some weird and esoteric formats, but maybe doing like... csv is cool too I guess.
Get in touch if you have a particular need, and we can see whether we can pull some programming crimes together.

In other words: contributions and collaboration welcome.

## Dependencies

Depends on the [`terminal_size`](https://crates.io/crates/terminal_size) crate for decoding and encoding many formats.
No other dependencies.

---

By [Ma3ke](https://hachyderm.io/@ma3ke).

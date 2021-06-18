# Dice Roller Which Needs A Better Name

When finished, this program will be a highly versatile and powerful dice roller, inspired by the iOS app Dice Calculator. At the moment, however, it's in the middle of a major code rework, and will likely require a lot more restructuring before it's worth using for most people.

The central intended selling points are:
- Versatile user interface, available both in GUI and CLI formats
- Fully offline; no need to rely on a Discord dice bot, or on [that ancient Wizards dice roller](https://www.wizards.com/dnd/dice/dice.htm), or any similar bit of potentially-rottable online infrastructure
- Multi-platform support, bringing Rust's ease of cross-compilation to bear

TODOs:
- Improve whitespace handling so as to avoid the current lossiness
- Add options for variable display verbosity, so as to be more easily pipable
- Improve format functions in `dice.rs`
- Figure out an elegant way to handle verbose display of nested dice rolls. (e.g. 2d3d4. Rendering as (2, 1)d4 loses something; so does rendering as (4, 2, 4).)
- Implement GUI

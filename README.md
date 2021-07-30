# Fluorite

When finished, this program will be a highly versatile and powerful dice roller, inspired by the iOS app Dice Calculator. At the moment, its core functionality is in place, but it suffers from a dramatic lack of polish.

The central intended selling points for the program's release are:
- Versatile user interface, available both in GUI and CLI formats
- Fully offline; no need to rely on a Discord dice bot, or [that ancient Wizards dice roller](https://www.wizards.com/dnd/dice/dice.htm), or any similar bit of potentially-rottable online infrastructure
- Multi-platform support, bringing Rust's ease of cross-compilation to bear
- Calculator functionality, supporting a variety of operations freely mixable with die rolls

## TODOs

### Functionality (general)

- Figure out an elegant way to handle verbose display of nested dice rolls. (See, for example, `2d3d4`. Rendering as [2, 1]d4 loses something; so does rendering as [4, 2, 4].)
- Recognize preemptively and error when an input has a chance to try to roll a non-integer number of dice or a d[non-integer], rather than only when it actually does so

### Functionality (GUI)

- Allow pressing enter to work in place of button-pressing for shortcut-creation
- Allow reordering shortcuts after their creation
- Figure out text-wrapping for large inputs/outputs (in place of the current sideways scrollbar / nothing)
- Provide feedback on shortcut-creation failure (and also for roll errors in a non-history-clogging way, while I'm at it)
- Prettify the interface

### Code Prettiness
- Maybe find a way to make the binop-sequence-handling code in `parse.rs` less repetitive?
- Make the FormatValidationError type implementations not be the horrible hacks they currently are

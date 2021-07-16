# Fluorite

When finished, this program will be a highly versatile and powerful dice roller, inspired by the iOS app Dice Calculator. At the moment, its functionality is limited; but its CLI is already mostly functional, for those for whom that's enough.

The central intended selling points for the program's release are:
- Versatile user interface, available both in GUI and CLI formats
- Fully offline; no need to rely on a Discord dice bot, or on [that ancient Wizards dice roller](https://www.wizards.com/dnd/dice/dice.htm), or any similar bit of potentially-rottable online infrastructure
- Multi-platform support, bringing Rust's ease of cross-compilation to bear
- Calculator functionality, supporting a variety of operations freely mixable with die rolls

## TODOs

### Functionality

- Allow more fine-tuned variation in output verbosity
- Improve whitespace handling so as to avoid the current lossiness
- Figure out an elegant way to handle verbose display of nested dice rolls. (See, for example, `2d3d4`. Rendering as [2, 1]d4 loses something; so does rendering as [4, 2, 4].)
- Add support for multiple inputs in a single CLI run, via `cat` or suchlike
- Allow pressing enter to work in place of button-pressing for shortcut-creation
- Allow reordering shortcuts after their creation

### Debug

- Correctly display results for nested rolls, which at present isn't just *inelegant* but instead straight-up *wrong*.

### Code Prettiness
- Improve format functions in `dice.rs`
- Maybe find a way to make the binop-sequence-handling code less repetitive?
- Figure out a better way to handle OOO maybe?

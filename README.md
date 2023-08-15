# minefair
An infinite implementation of Minesweeper that runs in the terminal. It's also completely fair.

![minefair being played](https://github.com/LyricLy/minefair/assets/8314814/a072ca1b-927c-4c77-93d0-1d559a1cbf8a)

# Fairness
What does being fair mean? `minefair` doesn't generate mines ahead of time like most Minesweeper implementations. Instead, it only generates the *numbers*, and a solver evaluates the validity of your moves.
That allows some changes to be made to the rules of the game, as we can essentially "move around" the mines based on the player's choices. The game comes with a number of "judges"; algorithms that essentially decide where
the mines are actually placed. Judges make a decision when you click about whether there should be a mine under that tile.

There is also an optional flag that will ensure that the generated board is always solvable (i.e. that at any given point there will always be at least 1 tile that is guaranteed not to be a mine).

## List of judges
* `random` - Decides at random based on the probability of the tile being a mine. Acts like the original game.
* `global` - Accepts any move that has the best probability of being safe across the whole board, but any move with a worse probability will always be a mine.
* `local` - A more forgiving version of `global` and the default judge. Acts like `global`, but only considers *logical regions*: if some cells are completely logically indepedent from the cell being clicked (because they are separated by guaranteed mines), they are not considered for the purposes of finding the move with the best probability. While `global` may punish you for making a slightly risky click because there is a safe cell on the other side of the map, `local` is more lenient in these scenarios.
* `kind` - Accepts any move that has a mine probability of less than 1.
* `strict` - Only accepts moves that have a mine probability of 0.
* `kaboom` - Imitates the rules of [Kaboom](https://pwmarcz.pl/kaboom/), another fair Minesweeper implementation. If there are any tiles that are 100% safe, you must click one of those. Otherwise you can click any tile that isn't guaranteed to be a mine.

# Controls
* Use WASD to pan the camera. The scroll wheel changes the speed. You can also drag with the mouse to pan.
* Left click to reveal a tile.
* Right click flags a tile as a mine.
* Left clicking a number performs the *chording* action: if the amount of flags around that tile is equal to the number shown, it clears all unflagged tiles around it.
* After dying, you are in a mode which shows which tiles would have been safe to press. You can press `j` to show the exact risk levels of each tile as hexadecimal digits.
* Ctrl+S saves the game. This is also done automatically when closing the game, or after every click if `--autosave` is passed.

# Installation
```
; cargo install --git https://github.com/LyricLy/minefair
```

# Command line usage

`minefair [OPTIONS] [SAVE_PATH]`
e.g. `minefair --judge strict --density 0.3 --solvable --autosave`

## Flags
* `--judge`: Pick the judge to use.
* `--density`: The density of the mines, represented as a probability from 0 to 1.
* `--solvable`: Ensure solvability with only moves on squares that are guaranteed not to be mines. The game is still fair without this flag, but it will require probabilistic play.
* `--reset` Clear the save file and start from scratch.
* `--cheat`: See the output from the solver, revealing how safe each square is.
* `--autosave`: Save automatically after each click. The default is only to save on pressing Ctrl+S or closing the game.

The `--judge`, `--density` and `--solvable` flags will be ignored if the save file already exists.

## Saving
The positional SAVE_PATH argument can be used to set the path of the file to use for save data. It can also be set using the `MINEFAIR_SAVE` environment variable.
If neither of these are present, one of the following defaults is used:
- `$XDG_DATA_HOME/minefair/save.minefair` or `~/.local/share/minefair/save.minefair` (Linux)
- `~/Library/Application Support/minefair/save.minefair` (macOS)
- `%APPDATA%\minefair\save.minefair` (Windows)

Pressing Ctrl+S or closing the game with Ctrl+C or Esc will save the game. Revealing a tile will also save if the `--autosave` flag is passed.

# minefair
An infinite implementation of Minesweeper that runs in the terminal. It's also completely fair.

# Fairness
What does being fair mean? `minefair` doesn't generate mines ahead of time like most Minesweeper implementations. Instead, it only generates the *numbers*, and a solver evaluates the validity of your moves.
That allows some changes to be made to the rules of the game, as we can essentially "move around" the mines based on the player's choices. The game comes with a number of "judges"; algorithms that essentially decide where
the mines are actually placed. Judges make a decision when you click about whether there should be a mine under that tile.

There is also an optional flag that will ensure that the generated board is always solvable (i.e. that at any given point there will always be at least 1 tile that is guaranteed not to be a mine).

## List of judges
* `random` - Decides at random based on the probability of the tile being a mine. Acts like the original game.
* `global` - Accepts any move that has the best probability of being safe across the whole board, but any move with a worse probability will always be a mine.
* `threshold` - Accepts any move with a probability better than a specified threshold.
* `kaboom` - Imitates the rules of [Kaboom](https://pwmarcz.pl/kaboom/), another fair Minesweeper implementation. If there are any tiles that are 100% safe, you must click one of those. Otherwise you can click any tile that isn't guaranteed to be a mine.

# Controls
* Use WASD to pan the camera. The scroll wheel changes the speed. You can also drag with the mouse to pan.
* Left click to reveal a tile.
* Right click flags a tile as a mine.
* Left clicking a number performs the *chording* action: if the amount of flags around that tile is equal to the number shown, it clears all unflagged tiles around it.
* After dying, you are in a mode which shows which tiles would have been safe to press. You can press `j` to show the exact risk levels of each tile as hexadecimal digits.

# Installation
```
; git clone https://github.com/LyricLy/minefair
; cd minefair
; cargo install --path .
```

# Command line usage
Saving isn't supported yet.

Do `minefair` on its own to start a new game. You can modify game settings with the following flags:
* `--judge`: Pick the judge to use.
* `--density`: The density of the mines, represented as a probability from 0 to 1.
* `--solvable`: Ensure solvability without any moves that aren't 100% likely not to be mines.
* `--cheat`: See the output from the solver, revealing how safe each square is.

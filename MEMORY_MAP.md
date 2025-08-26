# TIC-80 tiny computer
1.1.2837 (be42d6f)
https://tic80.com (C) 2017-2023

## Welcome
TIC-80 is a fantasy computer for making, playing and sharing tiny games.

It has built-in tools for development: code, sprites, maps, sound editors and the command line, which is enough to create a mini retro game.
In the end, you will get a cartridge file, which can be stored and played on the website.

Also, the game can be packed into a player that works on all popular platforms and distributed as you wish.
To make a retro-style game, the whole creation process takes place under some technical limitations: 240x136 pixels display, 16 color palette, 256 8x8 color sprites, 4 channel sound, etc.

## Specification
```
DISPLAY   240x136 pixels, 16 colors palette.
INPUT     4 gamepads with 8 buttons / mouse / keyboard.
SPRITES   256 8x8 tiles and 256 8x8 sprites.
MAP       240x136 cells, 1920x1088 pixels.
SOUND     4 channels with configurable waveforms.
CODE      64KB of lua, ruby, js, moon, fennel, scheme, squirrel, wren, wasm, janet or python.
```
```

+-----------------------------------+
|           96KB RAM LAYOUT         |
+-------+-------------------+-------+
| ADDR  | INFO              | BYTES |
+-------+-------------------+-------+
| 00000 | <VRAM>            | 16384 |
| 04000 | TILES             | 8192  |
| 06000 | SPRITES           | 8192  |
| 08000 | MAP               | 32640 |
| 0FF80 | GAMEPADS          | 4     |
| 0FF84 | MOUSE             | 4     |
| 0FF88 | KEYBOARD          | 4     |
| 0FF8C | SFX STATE         | 16    |
| 0FF9C | SOUND REGISTERS   | 72    |
| 0FFE4 | WAVEFORMS         | 256   |
| 100E4 | SFX               | 4224  |
| 11164 | MUSIC PATTERNS    | 11520 |
| 13E64 | MUSIC TRACKS      | 408   |
| 13FFC | MUSIC STATE       | 4     |
| 14000 | STEREO VOLUME     | 4     |
| 14004 | PERSISTENT MEMORY | 1024  |
| 14404 | SPRITE FLAGS      | 512   |
| 14604 | FONT              | 1016  |
| 149FC | FONT PARAMS       | 8     |
| 14A04 | ALT FONT          | 1016  |
| 14DFC | ALT FONT PARAMS   | 8     |
| 14E04 | BUTTONS MAPPING   | 32    |
| 14E24 | ** RESERVED **    | 12764 |
+-------+-------------------+-------+
```
```
+-----------------------------------+
|          16KB VRAM LAYOUT         |
+-------+-------------------+-------+
| ADDR  | INFO              | BYTES |
+-------+-------------------+-------+
| 00000 | SCREEN            | 16320 |
| 03FC0 | PALETTE           | 48    |
| 03FF0 | PALETTE MAP       | 8     |
| 03FF8 | BORDER COLOR      | 1     |
| 03FF9 | SCREEN OFFSET     | 2     |
| 03FFB | MOUSE CURSOR      | 1     |
| 03FFC | BLIT SEGMENT      | 1     |
| 03FFD | ... (reserved)    | 3     |
+-------+-------------------+-------+
```

## Console commands

### cd
change directory.
usage: `
cd <path>
cd /
cd ..`

### cls
clear console screen.
usage: `cls`

### config
edit system configuration cartridge,
use `reset` param to reset current configuration,
use `default` to edit default cart template.
usage: `config [reset|default]`

### del
delete from the filesystem.
usage: `del <file|folder>`

### demo
install demo carts to the current directory.
usage: `demo`

### dir
show list of local files.
usage: `dir`

### edit
open cart editors.
usage: `edit`

### eval
run code provided code.
usage: `eval`

### exit
exit the application.
usage: `exit`

### export
export cart to HTML,
native build (win linux rpi mac),
export sprites/map/... as a .png image or export sfx and music to .wav files.
usage: `
export [win|winxp|linux|rpi|mac|html|binary|tiles|sprites|map|mapimg|sfx|music|screen|help|...] <file> [bank=0 vbank=0 id=0 ...]`

### folder
open working directory in OS.
usage: `folder`

### help
show help info about commands/api/...
usage: `help [<text>|version|welcome|spec|ram|vram|commands|api|keys|buttons|startup|terms|license]`

### import
import code/sprites/map/... from an external file.
usage: `
import [binary|tiles|sprites|map|code|screen|...] <file> [bank=0 x=0 y=0 w=0 h=0 vbank=0 ...]`

### load
load cartridge from the local filesystem(there's no need to type the .tic extension).
you can also load just the section (sprites, map etc) from another cart.
usage: `load <cart> [code|tiles|sprites|map|sfx|music|palette|flags|screen]`

### menu
show game menu where you can setup video, sound and input options.
usage: `menu`

### mkdir
make a directory.
usage: `mkdir <name>`

### new
creates a new `Hello World` cartridge.
usage: `new <lua|ruby|js|moon|fennel|scheme|squirrel|wren|wasm|janet|python>`

### resume
resume last run cart / project.
usage: `resume`

### run
run current cart / project.
usage: `run`

### save
save cartridge to the local filesystem, use .lua .rb .js .moon .fnl .scm .nut .wren .wasmp .janet .py cart extension to save it in text format (PRO feature).
usage: `save <cart>`

### surf
open carts browser.
usage: `surf`

## API functions

### BDR
`BDR(row)`
Allows you to execute code between the drawing of each fullscreen scanline, for example, to manipulate the palette.

### BOOT
`BOOT`
Startup function.

### MENU
`MENU(index)`
Game Menu handler.

### SCN
`SCN(row)`
Allows you to execute code between the drawing of each scanline, for example, to manipulate the palette.

### TIC
`TIC()`
Main function. It's called at 60fps (60 times every second).

### btn
`btn(id) -> pressed`
This function allows you to read the status of one of the buttons attached to TIC.
The function returns true if the key with the supplied id is currently in the pressed state.
It remains true for as long as the key is held down.
If you want to test if a key was just pressed, use `btnp()` instead.

### btnp
`btnp(id hold=-1 period=-1) -> pressed`
This function allows you to read the status of one of TIC's buttons.
It returns true only if the key has been pressed since the last frame.
You can also use the optional hold and period parameters which allow you to check if a button is being held down.
After the time specified by hold has elapsed, btnp will return true each time period is passed if the key is still down.
For example, to re-examine the state of button `0` after 2 seconds and continue to check its state every 1/10th of a second, you would use btnp(0, 120, 6).
Since time is expressed in ticks and TIC runs at 60 frames per second, we use the value of 120 to wait 2 seconds and 6 ticks (ie 60/10) as the interval for re-checking.

### circ
`circ(x y radius color)`
This function draws a filled circle of the desired radius and color with its center at x, y.
It uses the Bresenham algorithm.

### circb
`circb(x y radius color)`
Draws the circumference of a circle with its center at x, y using the radius and color requested.
It uses the Bresenham algorithm.

### clip
`clip(x y width height)
clip()`
This function limits drawing to a clipping region or `viewport` defined by x,y,w,h.
Things drawn outside of this area will not be visible.
Calling clip() with no parameters will reset the drawing area to the entire screen.

### cls
`cls(color=0)`
Clear the screen.
When called this function clear all the screen using the color passed as argument.
If no parameter is passed first color (0) is used.

### elli
`elli(x y a b color)`
This function draws a filled ellipse of the desired a, b radiuses and color with its center at x, y.
It uses the Bresenham algorithm.

### ellib
`ellib(x y a b color)`
This function draws an ellipse border with the desired radiuses a b and color with its center at x, y.
It uses the Bresenham algorithm.

### exit
`exit()`
Interrupts program execution and returns to the console when the TIC function ends.

### fget
`fget(sprite_id flag) -> bool`
Returns true if the specified flag of the sprite is set. See `fset()` for more details.

### font
`font(text x y chromakey char_width char_height fixed=false scale=1 alt=false) -> width`
Print string with font defined in foreground sprites.
To simply print to the screen, check out `print()`.
To print to the console, check out `trace()`.

### fset
`fset(sprite_id flag bool)`
Each sprite has eight flags which can be used to store information or signal different conditions.
For example, flag 0 might be used to indicate that the sprite is invisible, flag 6 might indicate that the flag should be draw scaled etc.
See algo `fget()`.

### key
`key(code=-1) -> pressed`
The function returns true if the key denoted by keycode is pressed.

### keyp
`keyp(code=-1 hold=-1 period=-1) -> pressed`
This function returns true if the given key is pressed but wasn't pressed in the previous frame.
Refer to `btnp()` for an explanation of the optional hold and period parameters.

### line
`line(x0 y0 x1 y1 color)`
Draws a straight line from point (x0,y0) to point (x1,y1) in the specified color.

### map
`map(x=0 y=0 w=30 h=17 sx=0 sy=0 colorkey=-1 scale=1 remap=nil)`
The map consists of cells of 8x8 pixels, each of which can be filled with a sprite using the map editor.
The map can be up to 240 cells wide by 136 deep.
This function will draw the desired area of the map to a specified screen position.
For example, map(5,5,12,10,0,0) will draw a 12x10 section of the map, starting from map coordinates (5,5) to screen position (0,0).
The map function's last parameter is a powerful callback function for changing how map cells (sprites) are drawn when map is called.
It can be used to rotate, flip and replace sprites while the game is running.
Unlike mset, which saves changes to the map, this special function can be used to create animated tiles or replace them completely.
Some examples include changing sprites to open doorways, hiding sprites used to spawn objects in your game and even to emit the objects themselves.
The tilemap is laid out sequentially in RAM - writing 1 to 0x08000 will cause tile(sprite) #1 to appear at top left when map() is called.
To set the tile immediately below this we need to write to 0x08000 + 240, ie 0x080F0.

### memcpy
`memcpy(dest source size)`
This function allows you to copy a continuous block of TIC's 96K RAM from one address to another.
Addresses are specified are in hexadecimal format, values are decimal.

### memset
`memset(dest value size)`
This function allows you to set a continuous block of any part of TIC's RAM to the same value.
The address is specified in hexadecimal format, the value in decimal.

### mget
`mget(x y) -> tile_id`
Gets the sprite id at the given x and y map coordinate.

### mouse
`mouse() -> x y left middle right scrollx scrolly`
This function returns the mouse coordinates and a boolean value for the state of each mouse button,with true indicating that a button is pressed.

### mset
`mset(x y tile_id)`
This function will change the tile at the specified map coordinates.
By default, changes made are only kept while the current game is running.
To make permanent changes to the map, see `sync()`.
Related: `map()` `mget()` `sync()`.

### music
`music(track=-1 frame=-1 row=-1 loop=true sustain=false tempo=-1 speed=-1)`
This function starts playing a track created in the Music Editor.
Call without arguments to stop the music.

### peek
`peek(addr bits=8) -> value`
This function allows to read the memory from TIC.
It's useful to access resources created with the integrated tools like sprite, maps, sounds, cartridges data?
Never dream to sound a sprite?
Address are in hexadecimal format but values are decimal.
To write to a memory address, use `poke()`.
`bits` allowed to be 1,2,4,8.

### peek1
`peek1(addr) -> value`
This function enables you to read single bit values from TIC's RAM.
The address is often specified in hexadecimal format.

### peek2
`peek2(addr) -> value`
This function enables you to read two bits values from TIC's RAM.
The address is often specified in hexadecimal format.

### peek4
`peek4(addr) -> value`
This function enables you to read values from TIC's RAM.
The address is often specified in hexadecimal format.
See 'poke4()' for detailed information on how nibble addressing compares with byte addressing.

### pix
`pix(x y color)
pix(x y) -> color`
This function can read or write pixel color values.
When called with a color parameter, the pixel at the specified coordinates is set to that color.
Calling the function without a color parameter returns the color of the pixel at the specified position.

### pmem
`pmem(index value)
pmem(index) -> value`
This function allows you to save and retrieve data in one of the 256 individual 32-bit slots available in the cartridge's persistent memory.
This is useful for saving high-scores, level advancement or achievements.
The data is stored as unsigned 32-bit integers (from 0 to 4294967295).

Tips:
- pmem depends on the cartridge hash (md5), so don't change your lua script if you want to keep the data.
- Use `saveid:` with a personalized string in the header metadata to override the default MD5 calculation.
This allows the user to update a cart without losing their saved data.

### poke
`poke(addr value bits=8)`
This function allows you to write a single byte to any address in TIC's RAM.
The address should be specified in hexadecimal format, the value in decimal.
`bits` allowed to be 1,2,4,8.

### poke1
`poke1(addr value)`
This function allows you to write single bit values directly to RAM.
The address is often specified in hexadecimal format.

### poke2
`poke2(addr value)`
This function allows you to write two bits values directly to RAM.
The address is often specified in hexadecimal format.

### poke4
`poke4(addr value)`
This function allows you to write directly to RAM.
The address is often specified in hexadecimal format.
For both peek4 and poke4 RAM is addressed in 4 bit segments (nibbles).
Therefore, to access the the RAM at byte address 0x4000
you would need to access both the 0x8000 and 0x8001 nibble addresses.

### print
`print(text x=0 y=0 color=15 fixed=false scale=1 smallfont=false) -> width`
This will simply print text to the screen using the font defined in config.
When set to true, the fixed width option ensures that each character will be printed in a `box` of the same size, so the character `i` will occupy the same width as the character `w` for example.
When fixed width is false, there will be a single space between each character.

Tips:
- To use a custom rastered font, check out `font()`.
- To print to the console, check out `trace()`.

### rect
`rect(x y w h color)`
This function draws a filled rectangle of the desired size and color at the specified position.
If you only need to draw the the border or outline of a rectangle (ie not filled) see `rectb()`.

### rectb
`rectb(x y w h color)`
This function draws a one pixel thick rectangle border at the position requested.
If you need to fill the rectangle with a color, see `rect()` instead.

### reset
`reset()`
Resets the cartridge. To return to the console, see the `exit()`.

### sfx
`sfx(id note=-1 duration=-1 channel=0 volume=15 speed=0)`
This function will play the sound with `id` created in the sfx editor.
Calling the function with id set to -1 will stop playing the channel.
The note can be supplied as an integer between 0 and 95 (representing 8 octaves of 12 notes each) or as a string giving the note name and octave.
For example, a note value of `14` will play the note `D` in the second octave.
The same note could be specified by the string `D-2`.
Note names consist of two characters, the note itself (in upper case) followed by `-` to represent the natural note or `#` to represent a sharp.
There is no option to indicate flat values.
The available note names are therefore: C-, C#, D-, D#, E-, F-, F#, G-, G#, A-, A#, B-.
The `octave` is specified using a single digit in the range 0 to 8.
The `duration` specifies how many ticks to play the sound for since TIC-80 runs at 60 frames per second, a value of 30 represents half a second.
A value of -1 will play the sound continuously.
The `channel` parameter indicates which of the four channels to use. Allowed values are 0 to 3.
The `volume` can be between 0 and 15.
The `speed` in the range -4 to 3 can be specified and means how many `ticks+1` to play each step, so speed==0 means 1 tick per step.

### spr
`spr(id x y colorkey=-1 scale=1 flip=0 rotate=0 w=1 h=1)`
Draws the sprite number index at the x and y coordinate.
You can specify a colorkey in the palette which will be used as the transparent color or use a value of -1 for an opaque sprite.
The sprite can be scaled up by a desired factor. For example, a scale factor of 2 means an 8x8 pixel sprite is drawn to a 16x16 area of the screen.
You can flip the sprite where:
- 0 = No Flip
- 1 = Flip horizontally
- 2 = Flip vertically
- 3 = Flip both vertically and horizontally
When you rotate the sprite, it's rotated clockwise in 90 steps:
- 0 = No rotation
- 1 = 90 rotation
- 2 = 180 rotation
- 3 = 270 rotation
You can draw a composite sprite (consisting of a rectangular region of sprites from the sprite sheet) by specifying the `w` and `h` parameters (which default to 1).

### sync
`sync(mask=0 bank=0 tocart=false)`
The pro version of TIC-80 contains 8 memory banks.
To switch between these banks, sync can be used to either load contents from a memory bank to runtime, or save contents from the active runtime to a bank.
The function can only be called once per frame.If you have manipulated the runtime memory (e.g. by using mset), you can reset the active state by calling sync(0,0,false).
This resets the whole runtime memory to the contents of bank 0.Note that sync is not used to load code from banks; this is done automatically.

### time
`time() -> ticks`
This function returns the number of milliseconds elapsed since the cartridge began execution.
Useful for keeping track of time, animating items and triggering events.

### trace
`trace(message color=15)`
This is a service function, useful for debugging your code.
It prints the message parameter to the console in the (optional) color specified.

Tips:
- The Lua concatenator for strings is .. (two points).
- Use console cls command to clear the output from trace.

### tri
`tri(x1 y1 x2 y2 x3 y3 color)`
This function draws a triangle filled with color, using the supplied vertices.

### trib
`trib(x1 y1 x2 y2 x3 y3 color)`
This function draws a triangle border with color, using the supplied vertices.

### tstamp
`tstamp() -> timestamp`
This function returns the number of seconds elapsed since January 1st, 1970.
Useful for creating persistent games which evolve over time between plays.

### ttri
`ttri(x1 y1 x2 y2 x3 y3 u1 v1 u2 v2 u3 v3 texsrc=0 chromakey=-1 z1=0 z2=0 z3=0)`
It renders a triangle filled with texture from image ram, map ram or vbank.
Use in 3D graphics.
In particular, if the vertices in the triangle have different 3D depth, you may see some distortion.
These can be thought of as the window inside image ram (sprite sheet), map ram or another vbank.
Note that the sprite sheet or map in this case is treated as a single large image, with U and V addressing its pixels directly, rather than by sprite ID.
So for example the top left corner of sprite #2 would be located at u=16, v=0.

### vbank
`vbank(bank) -> prev
vbank() -> prev`
VRAM contains 2x16K memory chips, use vbank(0) or vbank(1) to switch between them.

## Button IDs
```
+--------+----+----+----+----+
| ACTION | P1 | P2 | P3 | P4 |
+--------+----+----+----+----+
|     UP |  0 |  8 | 16 | 24 |
|   DOWN |  1 |  9 | 17 | 25 |
|   LEFT |  2 | 10 | 18 | 26 |
|  RIGHT |  3 | 11 | 19 | 27 |
|      A |  4 | 12 | 20 | 28 |
|      B |  5 | 13 | 21 | 29 |
|      X |  6 | 14 | 22 | 30 |
|      Y |  7 | 15 | 23 | 31 |
+--------+----+----+----+----+```

## Key IDs
```
+----+------------+ +----+------------+
|CODE|    KEY     | |CODE|    KEY     |
+----+------------+ +----+------------+
|  1 | A          | | 48 | SPACE      |
|  2 | B          | | 49 | TAB        |
|  3 | C          | | 50 | RETURN     |
|  4 | D          | | 51 | BACKSPACE  |
|  5 | E          | | 52 | DELETE     |
|  6 | F          | | 53 | INSERT     |
|  7 | G          | | 54 | PAGEUP     |
|  8 | H          | | 55 | PAGEDOWN   |
|  9 | I          | | 56 | HOME       |
| 10 | J          | | 57 | END        |
| 11 | K          | | 58 | UP         |
| 12 | L          | | 59 | DOWN       |
| 13 | M          | | 60 | LEFT       |
| 14 | N          | | 61 | RIGHT      |
| 15 | O          | | 62 | CAPSLOCK   |
| 16 | P          | | 63 | CTRL       |
| 17 | Q          | | 64 | SHIFT      |
| 18 | R          | | 65 | ALT        |
| 19 | S          | | 66 | ESC        |
| 20 | T          | | 67 | F1         |
| 21 | U          | | 68 | F2         |
| 22 | V          | | 69 | F3         |
| 23 | W          | | 70 | F4         |
| 24 | X          | | 71 | F5         |
| 25 | Y          | | 72 | F6         |
| 26 | Z          | | 73 | F7         |
| 27 | 0          | | 74 | F8         |
| 28 | 1          | | 75 | F9         |
| 29 | 2          | | 76 | F10        |
| 30 | 3          | | 77 | F11        |
| 31 | 4          | | 78 | F12        |
| 32 | 5          | | 79 | NUM0       |
| 33 | 6          | | 80 | NUM1       |
| 34 | 7          | | 81 | NUM2       |
| 35 | 8          | | 82 | NUM3       |
| 36 | 9          | | 83 | NUM4       |
| 37 | MINUS      | | 84 | NUM5       |
| 38 | EQUALS     | | 85 | NUM6       |
| 39 | LEFTBRACKET| | 86 | NUM7       |
| 40 | RIGHTBRACKT| | 87 | NUM8       |
| 41 | BACKSLASH  | | 88 | NUM9       |
| 42 | SEMICOLON  | | 89 | NUMPLUS    |
| 43 | APOSTROPHE | | 90 | NUMMINUS   |
| 44 | GRAVE      | | 91 | NUMMULTIPLY|
| 45 | COMMA      | | 92 | NUMDIVIDE  |
| 46 | PERIOD     | | 93 | NUMENTER   |
| 47 | SLASH      | | 94 | NUMPERIOD  |
+----+------------+ +----+------------+
```

## Startup options
```
--skip           skip startup animation
--volume=<int>   global volume value [0-15]
--cli            console only output
--fullscreen     enable fullscreen mode
--vsync          enable VSYNC
--soft           use software rendering
--fs=<str>       path to the file system folder
--scale=<int>    main window scale
--cmd=<str>      run commands in the console
--keepcmd        re-execute commands on every run
--version        print program version
--crt            enable CRT monitor effect
```

## Terms of Use
- All cartridges posted on the https://tic80.com website are the property of their authors.
- Do not redistribute the cartridge without permission, directly from the author.
- By uploading cartridges to the site, you grant Nesbox the right to freely use and distribute them. All other rights by default remain with the author.
- Do not post material that violates copyright, obscenity or any other laws.
- Nesbox reserves the right to remove or filter any material without prior notice.

## Privacy Policy
We store only the user's email and password in encrypted form and will not transfer any personal information to third parties without explicit permission.

## MIT License

Copyright (c) 2017-2023 Vadim Grigoruk @nesbox // grigoruk@gmail.com

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the 'Software'), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
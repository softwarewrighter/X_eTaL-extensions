# The terminal part of game-assets.webm (videos/game-assets.film joins
# it to Blender's render): the program, one piece in X_eTaL, and the
# twelve made by Blender with their clips read back.
Output "target/screens/blender-game-assets-intro.mp4"
Set Shell bash
Set FontSize 14
Set FontFamily "JuliaMono"
Set Width 1280
Set Height 720
Set Padding 16
Set Framerate 12
Hide
Type "cd extensions/blender && clear"
Enter
Show
Type "sed -n '/^## Blender, 4/,/^## Run/p' demos/game-assets.xtl"
Sleep 300ms
Enter
Sleep 5s
Type "clear; sed -n '/^h:a_rrow1/,/^}/p;/^h:a_imKeys/,/^}/p' demos/Td.xtl"
Sleep 300ms
Enter
Sleep 7s
Type "clear; just demo td-models 2>&1 | tail -32"
Sleep 300ms
Enter
Sleep 9s

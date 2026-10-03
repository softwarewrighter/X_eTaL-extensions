# extensions/sqlite/videos/notebook.webm and .webp: the data notebook (CO2 at Mauna Loa) at the command line
# (just videos; vhs renders headlessly, no window opens).
Output "target/screens/sqlite-notebook.mp4"
Set Shell bash
Set FontSize 16
Set FontFamily "JuliaMono"
Set Width 1100
Set Height 720
Set Padding 16
Set Framerate 12
Hide
Type "clear"
Enter
Show
Type "just demo sqlite notebook --echo --seed 1 --delay 300"
Sleep 500ms
Enter
Sleep 30s

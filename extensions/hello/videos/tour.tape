# extensions/hello/videos/tour.webm and .webp: hello's tour at the command line
# (just videos; vhs renders headlessly, no window opens).
Output "target/screens/hello-tour.mp4"
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
Type "just demo hello tour --echo --seed 1 --delay 300"
Sleep 500ms
Enter
Sleep 12s

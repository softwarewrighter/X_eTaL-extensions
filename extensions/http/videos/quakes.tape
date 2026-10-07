# extensions/http/videos/quakes.webm and .webp: the earthquake report (the saved USGS feed) at the command line
# (just videos; vhs renders headlessly, no window opens).
Output "target/screens/http-quakes.mp4"
Set Shell bash
Set FontSize 16
Set FontFamily "JuliaMono"
Set Width 1100
Set Height 820
Set Padding 16
Set Framerate 12
Hide
Type "clear"
Enter
Show
Type "just demo http quakes --echo --seed 1 --delay 400"
Sleep 500ms
Enter
Sleep 14s

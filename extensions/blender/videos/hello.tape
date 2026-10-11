# extensions/blender/videos/hello.webm and .webp: X_eTaL running Blender
# in the background at the command line (just videos; vhs renders
# headlessly, no window opens).
Output "target/screens/blender-hello.mp4"
Set Shell bash
Set FontSize 16
Set FontFamily "JuliaMono"
Set Width 1100
Set Height 560
Set Padding 16
Set Framerate 12
Hide
Type "clear"
Enter
Show
Type "just demo blender hello --echo"
Sleep 500ms
Enter
Sleep 6s

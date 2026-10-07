# extensions/scene/videos/voxels-chunk.webm and .webp: voxels 1, a chunk as an array, at the command line
# (just videos; vhs renders headlessly, no window opens).
Output "target/screens/scene-voxels-chunk.mp4"
Set Shell bash
Set FontSize 16
Set FontFamily "JuliaMono"
Set Width 1100
Set Height 900
Set Padding 16
Set Framerate 12
Hide
Type "clear"
Enter
Show
Type "just demo scene voxels-chunk --echo --seed 1 --delay 300"
Sleep 500ms
Enter
Sleep 16s

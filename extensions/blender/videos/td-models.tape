# extensions/blender/videos/td-models.webm and .webp: X_eTaL having
# Blender make the twelve tower defense pieces as .glb files with their
# clips, read back from each file (just videos; vhs renders headlessly,
# no window opens).
Output "target/screens/blender-td-models.mp4"
Set Shell bash
Set FontSize 14
Set FontFamily "JuliaMono"
Set Width 1280
Set Height 720
Set Padding 16
Set Framerate 12
Hide
Type "clear"
Enter
Show
Type "just demo blender td-models 2>&1 | tail -40"
Sleep 500ms
Enter
Sleep 9s

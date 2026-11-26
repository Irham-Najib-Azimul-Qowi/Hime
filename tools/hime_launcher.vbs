' tools\hime_launcher.vbs - Silent Launcher for Hime Ecosystem
' Completely prevents console / terminal window flashing on start/stop/shortcuts.
' Uses WScript.Shell.Run with SW_HIDE (0) so ZERO black console windows ever flicker.

Option Explicit

Dim objShell, objFSO, scriptDir, himeRoot, psExe, targetScript, cmd, action, i

Set objShell = CreateObject("WScript.Shell")
Set objFSO = CreateObject("Scripting.FileSystemObject")

' If executed via cscript.exe instead of wscript.exe, relaunch under wscript.exe so no console appears
If InStr(LCase(WScript.FullName), "cscript.exe") > 0 Then
    Dim wscriptBin, fullArgs, j
    wscriptBin = Replace(LCase(WScript.FullName), "cscript.exe", "wscript.exe")
    fullArgs = """" & WScript.ScriptFullName & """"
    For j = 0 To WScript.Arguments.Count - 1
        fullArgs = fullArgs & " """ & WScript.Arguments(j) & """"
    Next
    objShell.Run """" & wscriptBin & """ " & fullArgs, 0, False
    WScript.Quit
End If

scriptDir = objFSO.GetParentFolderName(WScript.ScriptFullName)
himeRoot = objFSO.GetParentFolderName(scriptDir)
psExe = objShell.ExpandEnvironmentStrings("%SystemRoot%") & "\System32\WindowsPowerShell\v1.0\powershell.exe"

If WScript.Arguments.Count > 0 Then
    action = LCase(WScript.Arguments(0))
    Select Case action
        Case "start"
            targetScript = himeRoot & "\scripts\start-hime.ps1"
            cmd = """" & psExe & """ -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File """ & targetScript & """"
            objShell.Run cmd, 0, False
        Case "start-autostart", "autostart"
            targetScript = himeRoot & "\scripts\start-hime.ps1"
            cmd = """" & psExe & """ -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File """ & targetScript & """ -Autostart"
            objShell.Run cmd, 0, False
        Case "stop", "quit"
            targetScript = himeRoot & "\scripts\stop-hime.ps1"
            cmd = """" & psExe & """ -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File """ & targetScript & """"
            objShell.Run cmd, 0, False
        Case Else
            ' If a file path or script was passed directly
            targetScript = WScript.Arguments(0)
            cmd = """" & psExe & """ -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File """ & targetScript & """"
            For i = 1 To WScript.Arguments.Count - 1
                cmd = cmd & " """ & WScript.Arguments(i) & """"
            Next
            objShell.Run cmd, 0, False
    End Select
Else
    targetScript = himeRoot & "\scripts\start-hime.ps1"
    cmd = """" & psExe & """ -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File """ & targetScript & """"
    objShell.Run cmd, 0, False
End If

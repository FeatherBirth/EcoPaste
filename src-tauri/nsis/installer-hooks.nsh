!macro StopEcoPasteProcesses
  DetailPrint "Stopping existing EcoPaste processes..."
  nsExec::ExecToLog 'taskkill /F /T /IM EcoPaste.exe'
  Pop $0
  Sleep 500
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro StopEcoPasteProcesses
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro StopEcoPasteProcesses
!macroend

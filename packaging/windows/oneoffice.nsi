Unicode true
ManifestDPIAware true

!include "MUI2.nsh"

!ifndef VERSION
  !error "pass /DVERSION=0.1.0"
!endif
!ifndef VI_VERSION
  !define VI_VERSION "0.0.0.0"
!endif
!ifndef OUTFILE
  !error "pass /DOUTFILE=dist\\OneOffice-setup.exe"
!endif
!ifndef SRC
  !error "pass /DSRC=target\\release\\oneoffice.exe"
!endif

Name "OneOffice ${VERSION}"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\OneOffice"
InstallDirRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "InstallLocation"
RequestExecutionLevel admin
ShowInstDetails show

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

VIProductVersion "${VI_VERSION}"
VIAddVersionKey "ProductName" "OneOffice"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "OneOffice"
VIAddVersionKey "LegalCopyright" "MIT OR Apache-2.0"

Section "OneOffice"
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "/oname=oneoffice.exe" "${SRC}"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateShortcut "$SMPROGRAMS\OneOffice.lnk" "$INSTDIR\oneoffice.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "DisplayName" "OneOffice ${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "Publisher" "OneOffice"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "UninstallString" "$\"$INSTDIR\Uninstall.exe$\""
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\oneoffice.exe"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$SMPROGRAMS\OneOffice.lnk"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneOffice"
SectionEnd

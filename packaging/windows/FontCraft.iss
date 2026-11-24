#ifndef AppVersion
#define AppVersion "0.0.0"
#endif

#ifndef SourceExe
#define SourceExe "..\..\target\release\fontcraft.exe"
#endif

[Setup]
AppId={{9D7C2C64-6E55-4C5B-8F52-3B9D5C0F7A11}
AppName=FontCraft
AppVersion={#AppVersion}
AppPublisher=mohseenjamall
AppPublisherURL=https://github.com/mohseenjamall/FontCraft
; The install folder must stay user-writable for the in-app updater.
PrivilegesRequired=lowest
DefaultDirName={localappdata}\Programs\FontCraft
DefaultGroupName=FontCraft
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=out
OutputBaseFilename=FontCraft-{#AppVersion}-windows-x64-setup
SetupIconFile=..\..\assets\icon.ico
UninstallDisplayIcon={app}\fontcraft.exe
LicenseFile=..\..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#AppVersion}

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\FontCraft"; Filename: "{app}\fontcraft.exe"; AppUserModelID: "mohseenjamall.FontCraft"
Name: "{autodesktop}\FontCraft"; Filename: "{app}\fontcraft.exe"; Tasks: desktopicon; AppUserModelID: "mohseenjamall.FontCraft"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Run]
Filename: "{app}\fontcraft.exe"; Description: "Launch FontCraft"; Flags: postinstall skipifsilent nowait

[UninstallDelete]
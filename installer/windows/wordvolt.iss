; Inno Setup script of the Windows installer. Built by .github/workflows/release.yml:
;   iscc /DAppVersion=0.1.0-alpha /DNumericVersion=0.1.0.0 installer\windows\wordvolt.iss
; The program is expected at dist\WordVolt.exe; the installer is written to dist\.

#ifndef AppVersion
  #define AppVersion "0.0.0-dev"
#endif
#ifndef NumericVersion
  #define NumericVersion "0.0.0.0"
#endif

[Setup]
AppId={{B6F0C3A1-5D5E-4C1B-9E0A-7A1B2C3D4E5F}
AppName=WordVolt
AppVersion={#AppVersion}
VersionInfoVersion={#NumericVersion}
AppPublisher=VoltCoreLabs
AppPublisherURL=https://github.com/VoltCoreLabs/WordVolt
AppSupportURL=https://github.com/VoltCoreLabs/WordVolt/issues
DefaultDirName={autopf}\WordVolt
DefaultGroupName=WordVolt
DisableProgramGroupPage=yes
OutputDir=..\..\dist
OutputBaseFilename=WordVolt-{#AppVersion}-windows-x64-setup
SetupIconFile=..\..\app\icon.ico
UninstallDisplayIcon={app}\WordVolt.exe
LicenseFile=..\..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associate"; Description: "Open .wvd documents with WordVolt"; GroupDescription: "File types:"

[Files]
Source: "..\..\dist\WordVolt.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\WordVolt"; Filename: "{app}\WordVolt.exe"
Name: "{autodesktop}\WordVolt"; Filename: "{app}\WordVolt.exe"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\.wvd"; ValueType: string; ValueData: "WordVolt.Document"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\WordVolt.Document"; ValueType: string; ValueData: "WordVolt document"; Flags: uninsdeletekey; Tasks: associate
Root: HKA; Subkey: "Software\Classes\WordVolt.Document\DefaultIcon"; ValueType: string; ValueData: "{app}\WordVolt.exe,0"; Tasks: associate
Root: HKA; Subkey: "Software\Classes\WordVolt.Document\shell\open\command"; ValueType: string; ValueData: """{app}\WordVolt.exe"" ""%1"""; Tasks: associate

[Run]
Filename: "{app}\WordVolt.exe"; Description: "{cm:LaunchProgram,WordVolt}"; Flags: nowait postinstall skipifsilent

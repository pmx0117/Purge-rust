; Native per-user installer. Keep AppId stable across releases.
[Setup]
AppId={{7080F5A4-FCF2-4F15-B2E7-73C29D00DA65}
AppName=Purge
AppVersion={#AppVersion}
AppVerName=Purge {#AppVersion}
AppPublisher=Purge
VersionInfoVersion={#AppNumericVersion}
VersionInfoProductName=Purge
DefaultDirName={localappdata}\Programs\Purge
DisableDirPage=no
DisableProgramGroupPage=yes
UninstallDisplayName=Purge
UninstallDisplayIcon={app}\purge.exe
SetupIconFile={#IconFile}
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
WizardStyle=modern dynamic
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
CloseApplicationsFilter=purge.exe
RestartApplications=no
SetupMutex=PurgeInstaller

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\purge.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{userprograms}\Purge"; Filename: "{app}\purge.exe"
Name: "{userdesktop}\Purge"; Filename: "{app}\purge.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\purge.exe"; Description: "{cm:LaunchProgram,Purge}"; Flags: nowait postinstall skipifsilent

[Setup]
AppId=Sinesi
AppName=Sinesi
AppVersion={#AppVersion}
DefaultDirName={localappdata}\Programs\Sinesi
DefaultGroupName=Sinesi
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes
OutputDir={#OutputDir}
OutputBaseFilename=Sinesi-{#AppVersion}-windows-x64
Compression=lzma2
SolidCompression=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "portuguese"; MessagesFile: "compiler:Languages\Portuguese.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"

[Files]
Source: "{#BinDir}\sns-desktop.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\sinesi.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\fonts\OFL.txt"; DestDir: "{app}"; DestName: "Noto-OFL.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\Sinesi"; Filename: "{app}\sns-desktop.exe"

[Registry]
Root: HKCU; Subkey: "Software\Classes\.sinesi\OpenWithProgids"; ValueType: string; ValueName: "Sinesi.Project"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\Sinesi.Project"; ValueType: string; ValueData: "Sinesi"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Sinesi.Project\shell\open\command"; ValueType: string; ValueData: """{app}\sns-desktop.exe"" -- ""%1"""
Root: HKCU; Subkey: "Software\Sinesi\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Sinesi"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Sinesi\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Sinesi"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Sinesi\Capabilities\FileAssociations"; ValueType: string; ValueName: ".sinesi"; ValueData: "Sinesi.Project"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "Sinesi"; ValueData: "Software\Sinesi\Capabilities"; Flags: uninsdeletevalue

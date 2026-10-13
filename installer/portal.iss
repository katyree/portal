#ifndef AppVersion
  #error AppVersion must be provided by build-installer.ps1
#endif
#ifndef PortalExecutable
  #error PortalExecutable must be provided by build-installer.ps1
#endif
#ifndef RepositoryRoot
  #error RepositoryRoot must be provided by build-installer.ps1
#endif
#ifndef InstallerOutput
  #error InstallerOutput must be provided by build-installer.ps1
#endif

[Setup]
AppId={{C471416E-8E08-4911-A7D1-F4390BB36E7C}
AppName=Portal
AppVersion={#AppVersion}
AppPublisher=Portal
AppPublisherURL=https://github.com/katyree/portal
AppSupportURL=https://github.com/katyree/portal/issues
DefaultDirName={localappdata}\Programs\Portal
DefaultGroupName=Portal
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.19041
DisableProgramGroupPage=auto
AllowNoIcons=yes
WizardStyle=modern
OutputDir={#InstallerOutput}
OutputBaseFilename=portal-setup
Compression=lzma2
SolidCompression=yes
Uninstallable=yes
UninstallDisplayIcon={app}\portal.exe
UninstallDisplayName=Portal
CloseApplications=yes
CloseApplicationsFilter=portal.exe
RestartApplications=no
SetupLogging=yes
SetupIconFile={#RepositoryRoot}\assets\portal.ico
VersionInfoVersion={#AppVersion}

[Tasks]
Name: desktopicon; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "{#PortalExecutable}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepositoryRoot}\third-party\GPUI-APACHE-2.0.txt"; DestDir: "{app}\licenses"; Flags: ignoreversion

[Icons]
Name: "{group}\Portal"; Filename: "{app}\portal.exe"; WorkingDir: "{app}"
Name: "{group}\Uninstall Portal"; Filename: "{uninstallexe}"; IconFilename: "{app}\portal.exe"
Name: "{autodesktop}\Portal"; Filename: "{app}\portal.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\portal.exe"; Description: "Open Portal"; Flags: nowait postinstall skipifsilent

; Inno Setup removes only logged files and shortcuts. Do not add recursive
; [UninstallDelete] entries: user files and LocalAppData\Portal settings stay intact.

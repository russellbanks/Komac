#define AppName "Komac"
#define Version GetFileProductVersion(InputExecutable)
#define Publisher "Russell Banks (鸭鸭「カモ」翻译)"
#define URL "https://github.com/DuckDuckStudio/Komac"
#define ExeName GetFileOriginalFilename(InputExecutable)

#if Pos("x64", Architecture) > 0
  #define ArchAllowed "x64compatible and not arm64"
#else
  #define ArchAllowed Architecture
#endif

#include "CodeDependencies.iss"

[Setup]
AppId={{776938BF-CF8E-488B-A3DF-8048BC64F2CD}
AppName={#AppName}
AppVersion={#Version}
AppPublisher={#Publisher}
AppPublisherURL={#URL}
AppSupportURL={#URL}
AppUpdatesURL={#URL}
DefaultDirName={autopf}\{#AppName}
DisableDirPage=yes
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
LicenseFile=gpl-3.0.rst
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputBaseFilename={#AppName}Setup-{#Version}-{#Architecture}
SetupIconFile=logo.ico
UninstallDisplayName={#AppName} ({#Architecture})
WizardStyle=modern
ChangesEnvironment=yes
ArchitecturesAllowed={#ArchAllowed}
ArchitecturesInstallIn64BitMode={#ArchAllowed}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Files]
Source: "{#InputExecutable}"; DestDir: "{app}\bin"; DestName: "{#ExeName}"

[Code]
function InitializeSetup: Boolean;
begin
  Dependency_AddVC2015To2022;
  Result := True;
end;

procedure EnvAddPath(Path: string);
var
    Paths: string;
    RootKey: Integer;
    EnvironmentKey: string;
begin
    if IsAdminInstallMode() then
    begin
        EnvironmentKey := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
        RootKey := HKEY_LOCAL_MACHINE;
    end
    else
    begin
        EnvironmentKey := 'Environment';
        RootKey := HKEY_CURRENT_USER;
    end;

    { Retrieve current path (use empty string if entry not exists) }
    if not RegQueryStringValue(RootKey, EnvironmentKey, 'Path', Paths)
    then Paths := '';

    { Skip if string already found in path }
    if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

    { App string to the end of the path variable }
    Paths := Paths + ';'+ Path +';'

    { Overwrite (or create if missing) path environment variable }
    if RegWriteStringValue(RootKey, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] added to PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while adding the [%s] to PATH: [%s]', [Path, Paths]));
end;


procedure EnvRemovePath(Path: string);
var
    Paths: string;
    P: Integer;
    RootKey: Integer;
    EnvironmentKey: string;
begin
    if Pos(ExpandConstant('{commonpf}'), ExpandConstant('{app}')) = 1 then
    begin
        EnvironmentKey := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
        RootKey := HKEY_LOCAL_MACHINE;
    end
    else
    begin
        EnvironmentKey := 'Environment';
        RootKey := HKEY_CURRENT_USER;
    end;

    { Skip if registry entry not exists }
    if not RegQueryStringValue(RootKey, EnvironmentKey, 'Path', Paths) then
        exit;

    { Skip if string not found in path }
    P := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';');
    if P = 0 then exit;

    { Update path variable }
    Delete(Paths, P - 1, Length(Path) + 1);

    { Overwrite path environment variable }
    if RegWriteStringValue(RootKey, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] removed from PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while removing the [%s] from PATH: [%s]', [Path, Paths]));
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
    if CurStep = ssPostInstall
     then EnvAddPath(ExpandConstant('{app}') +'\bin');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
    if CurUninstallStep = usPostUninstall
    then EnvRemovePath(ExpandConstant('{app}') +'\bin');
end;

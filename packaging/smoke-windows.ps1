param(
    [Parameter(Mandatory = $true)][string]$Package,
    [Parameter(Mandatory = $true)][string]$WorkDirectory,
    [switch]$Keyboard
)
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;

public static class SinesiShellSmoke {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct ShellExecuteInfo {
        public uint Size, Mask;
        public IntPtr Window;
        public string Verb, File, Parameters, Directory;
        public int Show;
        public IntPtr Instance, ItemList;
        public string ClassName;
        public IntPtr ClassKey;
        public uint HotKey;
        public IntPtr IconOrMonitor, ProcessHandle;
    }
    [DllImport("shell32.dll", ExactSpelling = true, SetLastError = true, CharSet = CharSet.Unicode)]
    private static extern bool ShellExecuteExW(ref ShellExecuteInfo info);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint GetProcessId(IntPtr process);
    [DllImport("kernel32.dll")]
    private static extern bool CloseHandle(IntPtr handle);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    public static Process Open(string project) {
        const uint ClassName = 0x00000001, NoCloseProcess = 0x00000040;
        const uint NoAsync = 0x00000100, NoErrorDialog = 0x00000400;
        var info = new ShellExecuteInfo {
            Size = (uint)Marshal.SizeOf(typeof(ShellExecuteInfo)),
            Mask = ClassName | NoCloseProcess | NoAsync | NoErrorDialog,
            Verb = "open", File = project, ClassName = "Sinesi.Project", Show = 1
        };
        if (!ShellExecuteExW(ref info)) throw new Win32Exception(Marshal.GetLastWin32Error());
        if (info.ProcessHandle == IntPtr.Zero) throw new InvalidOperationException("No shell process");
        try {
            uint pid = GetProcessId(info.ProcessHandle);
            if (pid == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
            return Process.GetProcessById((int)pid);
        } finally {
            CloseHandle(info.ProcessHandle);
        }
    }
}
'@

function Test-AssociatedWindow([string]$Project, [string]$Desktop, [string]$Exercise, $Case, $Catalog) {
    $settings = $env:SINESI_SETTINGS_PATH
    $sessions = $env:SINESI_SESSION_DIR
    try {
        $env:SINESI_SETTINGS_PATH = Join-Path $Exercise 'settings.json'
        $env:SINESI_SESSION_DIR = Join-Path $Exercise 'sessions'
        if ($Case) {
            $report = & $Desktop '--check-config' '--' $Project
            if ($LASTEXITCODE -ne 0) { throw "Could not check the keyboard fixture configuration ($LASTEXITCODE)" }
            $effective = (($report -join "`n") | ConvertFrom-Json).configuration.effective
            if ($effective.language -ne $Case.Language -or
                $effective.display.representation -ne $Case.View -or $effective.display.scale_percent -ne 150) {
                throw "Incorrect effective configuration for $($Case.Language)/$($Case.View)"
            }
        }
        # Select the registered handler without changing the user's default app.
        $process = [SinesiShellSmoke]::Open($Project)
    } finally {
        $env:SINESI_SETTINGS_PATH = $settings
        $env:SINESI_SESSION_DIR = $sessions
    }
    try {
        $command = (Get-CimInstance Win32_Process -Filter "ProcessId = $($process.Id)").CommandLine
        if ($process.MainModule.FileName -ne $Desktop -or
            -not $command.EndsWith((' -- "{0}"' -f $Project), [StringComparison]::OrdinalIgnoreCase)) {
            throw 'The shell did not launch the expected executable and project'
        }
        $deadline = [DateTime]::UtcNow.AddSeconds(20)
        do {
            $process.Refresh()
            if ($process.HasExited) { throw "Desktop exited ($($process.ExitCode))" }
            if ($process.MainWindowHandle -ne 0) {
                Write-Output 'OK — the Windows shell opened the associated project in Sinesi'
                if ($Case) { Test-DesktopKeyboard $process $Case $Catalog }
                return
            }
            Start-Sleep -Milliseconds 100
        } while ([DateTime]::UtcNow -lt $deadline)
        throw 'Sinesi did not show its main window'
    } finally {
        if (-not $process.HasExited) {
            $null = $process.CloseMainWindow()
            if (-not $process.WaitForExit(5000)) { $process.Kill(); $process.WaitForExit() }
        }
        $process.Dispose()
    }
}

$prog = 'HKCU:\Software\Classes\Sinesi.Project'
if (Test-Path $prog) { throw 'Sinesi is already registered for this user' }
$root = [IO.Path]::GetFullPath($WorkDirectory)
if (Test-Path -LiteralPath $root) { throw 'Use a new work directory for the installer test' }
if ($Keyboard) {
    if ([Threading.Thread]::CurrentThread.GetApartmentState() -ne 'STA') {
        throw 'Run the keyboard smoke test with Windows PowerShell in STA mode'
    }
    Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
    . (Join-Path $PSScriptRoot 'windows/KeyboardSmoke.ps1')
    $catalog = Get-Content -Raw -Encoding UTF8 -LiteralPath 'crates/sns-i18n/locales/messages.json' | ConvertFrom-Json
}
$install = Join-Path $root 'app'
$exercise = Join-Path $root 'exercise'
$installer = (Resolve-Path -LiteralPath $Package).Path
$setup = Start-Process -Wait -PassThru -FilePath $installer -ArgumentList @(
    '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', ('/DIR="{0}"' -f $install)
)
if ($setup.ExitCode -ne 0) { throw "Installer failed ($($setup.ExitCode))" }
try {
    $desktop = Join-Path $install 'sns-desktop.exe'
    $cli = Join-Path $install 'sinesi.exe'
    $expected = '"' + $desktop + '" -- "%1"'
    $registered = (Get-Item -LiteralPath "$prog\shell\open\command").GetValue('')
    if ($registered -ne $expected) { throw "Incorrect command: $registered" }
    $extension = 'HKCU:\Software\Classes\.sinesi\OpenWithProgids'
    if ((Get-Item -LiteralPath $extension).GetValue('Sinesi.Project', $null) -ne '') {
        throw 'The project association was not registered'
    }
    python packaging/smoke.py --desktop $desktop --cli $cli --exercise-dir $exercise
    if ($LASTEXITCODE -ne 0) { throw "Packaged binaries failed ($LASTEXITCODE)" }
    $project = Get-ChildItem -LiteralPath $exercise -Filter '*.sinesi' -File
    $originalHash = (Get-FileHash -LiteralPath $project.FullName).Hash
    Test-AssociatedWindow $project.FullName $desktop $exercise
    if ((Get-FileHash -LiteralPath $project.FullName).Hash -ne $originalHash) {
        throw 'Opening the associated project changed its contents'
    }
    if ($Keyboard) {
        foreach ($language in 'fr', 'en', 'es', 'it', 'pt', 'de') {
            foreach ($view in 'icons', 'diagram') {
                $directory = Join-Path $exercise "keyboard/$language/$view"
                $case = New-KeyboardCase $directory $language $view $project.Name
                Test-AssociatedWindow $case.Project $desktop $case.Directory $case $catalog
            }
        }
    }
    $fingerprints = Get-ChildItem -LiteralPath $exercise -Recurse -File -Force | Get-FileHash
} finally {
    $uninstall = Start-Process -Wait -PassThru -FilePath (Join-Path $install 'unins000.exe') -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART'
    )
    if ($uninstall.ExitCode -ne 0) { throw "Uninstaller failed ($($uninstall.ExitCode))" }
}
if ((Test-Path $prog) -or (Test-Path -LiteralPath $desktop)) { throw 'Uninstall left Sinesi registered or installed' }
$extensionKey = Get-Item -LiteralPath $extension -ErrorAction SilentlyContinue
if ($extensionKey -and $null -ne $extensionKey.GetValue('Sinesi.Project', $null)) {
    throw 'Uninstall left the Sinesi Open With entry'
}
foreach ($file in $fingerprints) {
    if ((Get-FileHash -LiteralPath $file.Path).Hash -ne $file.Hash) { throw "Uninstall changed $($file.Path)" }
}
Write-Output "OK - Sinesi uninstalled and $($fingerprints.Count) exercise files were preserved"

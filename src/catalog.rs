use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use sysinfo::{ProcessesToUpdate, System};
use windows::{
    Management::Deployment::PackageManager,
    Win32::{
        System::{
            Com::{CLSCTX_LOCAL_SERVER, CoCreateInstance},
            WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        },
        UI::{
            Shell::{
                AO_NONE, ApplicationActivationManager, IApplicationActivationManager, ShellExecuteW,
            },
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
    core::{HSTRING, PCWSTR, w},
};
use winreg::{RegKey, enums::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherId {
    Steam,
    Ubisoft,
    Epic,
    Xbox,
    Ea,
    Rockstar,
    BattleNet,
    Amazon,
}

impl LauncherId {
    pub const ALL: [Self; 8] = [
        Self::Steam,
        Self::Ubisoft,
        Self::Epic,
        Self::Xbox,
        Self::Ea,
        Self::Rockstar,
        Self::BattleNet,
        Self::Amazon,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Ubisoft => "Ubisoft Connect",
            Self::Epic => "Epic Games",
            Self::Xbox => "Xbox",
            Self::Ea => "EA",
            Self::Rockstar => "Rockstar Games",
            Self::BattleNet => "Battle.net",
            Self::Amazon => "Amazon Games",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            Self::Steam => "steam",
            Self::Ubisoft => "ubisoft",
            Self::Epic => "epicgames",
            Self::Xbox => "xbox",
            Self::Ea => "ea",
            Self::Rockstar => "rockstargames",
            Self::BattleNet => "battledotnet",
            Self::Amazon => "amazon",
        }
    }
    fn executables(self) -> &'static [&'static str] {
        match self {
            Self::Steam => &["steam.exe"],
            Self::Ubisoft => &["UbisoftConnect.exe", "upc.exe"],
            Self::Epic => &["EpicGamesLauncher.exe"],
            Self::Xbox => &["XboxPcApp.exe", "GamingApp.exe"],
            Self::Ea => &["EADesktop.exe"],
            Self::Rockstar => &["LauncherPatcher.exe", "Launcher.exe"],
            Self::BattleNet => &["Battle.net Launcher.exe", "Battle.net.exe"],
            Self::Amazon => &["Amazon Games.exe"],
        }
    }
    fn relative_paths(self) -> &'static [&'static str] {
        match self {
            Self::Steam => &[r"Steam\steam.exe"],
            Self::Ubisoft => &[
                r"Ubisoft\Ubisoft Game Launcher\UbisoftConnect.exe",
                r"Ubisoft\Ubisoft Game Launcher\upc.exe",
            ],
            Self::Epic => &[
                r"Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
                r"Epic Games\Launcher\Portal\Binaries\Win32\EpicGamesLauncher.exe",
            ],
            Self::Xbox => &[],
            Self::Ea => &[r"Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe"],
            Self::Rockstar => &[
                r"Rockstar Games\Launcher\LauncherPatcher.exe",
                r"Rockstar Games\Launcher\Launcher.exe",
            ],
            Self::BattleNet => &[
                r"Battle.net\Battle.net Launcher.exe",
                r"Battle.net\Battle.net.exe",
            ],
            Self::Amazon => &[r"Amazon Games\App\Amazon Games.exe"],
        }
    }
    fn matches_display_name(self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        match self {
            Self::Steam => name == "steam",
            Self::Ubisoft => name.contains("ubisoft connect") || name == "uplay",
            Self::Epic => name.contains("epic games launcher"),
            Self::Xbox => false,
            Self::Ea => name == "ea app" || name == "ea desktop",
            Self::Rockstar => name.contains("rockstar games launcher"),
            Self::BattleNet => name == "battle.net" || name.contains("battle.net launcher"),
            Self::Amazon => name == "amazon games" || name == "amazon games app",
        }
    }
}

#[derive(Clone, Debug)]
pub enum LaunchTarget {
    Executable(PathBuf),
    WindowsApp(String),
    XboxProtocol(&'static str),
}

#[derive(Clone, Debug)]
pub struct LauncherStatus {
    pub id: LauncherId,
    pub target: Option<LaunchTarget>,
    pub running: bool,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    #[serde(default)]
    executable_overrides: BTreeMap<LauncherId, PathBuf>,
    #[serde(default)]
    disabled_launchers: BTreeSet<LauncherId>,
}

#[derive(Clone)]
pub struct Catalog {
    settings_path: PathBuf,
    settings: Settings,
}

impl Catalog {
    pub fn load() -> Result<Self> {
        let directory = env::var_os("PORTAL_SETTINGS_DIR")
            .map(PathBuf::from)
            .or_else(|| env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Portal")))
            .context("Windows local app data directory is unavailable")?;
        Self::load_at(directory.join("settings.json"))
    }
    fn load_at(settings_path: PathBuf) -> Result<Self> {
        let settings = match fs::read(&settings_path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .context("Portal settings could not be read. The existing file was preserved.")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            settings_path,
            settings,
        })
    }
    pub fn scan(&self) -> Vec<LauncherStatus> {
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);
        LauncherId::ALL
            .into_iter()
            .filter(|id| self.is_enabled(*id))
            .map(|id| {
                let target = self.discover(id);
                let running = system.processes().values().any(|process| {
                    let mut known_process = id
                        .executables()
                        .iter()
                        .any(|exe| process.name().to_string_lossy().eq_ignore_ascii_case(exe));
                    if id == LauncherId::Rockstar && process.name().to_string_lossy().eq_ignore_ascii_case("Launcher.exe") {
                        known_process = matches!(&target, Some(LaunchTarget::Executable(launcher)) if process.exe().is_some_and(|exe| exe.parent() == launcher.parent()));
                    }
                    let override_process = self
                        .settings
                        .executable_overrides
                        .get(&id)
                        .is_some_and(|path| process.exe().is_some_and(|exe| exe == path));
                    known_process || override_process
                });
                LauncherStatus {
                    id,
                    target,
                    running,
                }
            })
            .collect()
    }
    fn discover(&self, id: LauncherId) -> Option<LaunchTarget> {
        if let Some(path) = self.settings.executable_overrides.get(&id) {
            return validate_executable(path).ok().map(LaunchTarget::Executable);
        }
        if id == LauncherId::Xbox {
            return discover_xbox();
        }
        let mut candidates = Vec::new();
        if id == LauncherId::Steam {
            for (hive, key, value) in [
                (HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath"),
                (
                    HKEY_LOCAL_MACHINE,
                    r"SOFTWARE\WOW6432Node\Valve\Steam",
                    "InstallPath",
                ),
            ] {
                if let Ok(key) = RegKey::predef(hive).open_subkey(key)
                    && let Ok(path) = key.get_value::<String, _>(value)
                {
                    candidates.push(PathBuf::from(path).join("steam.exe"));
                }
            }
        }
        if id == LauncherId::Ubisoft {
            for key in [
                r"SOFTWARE\Ubisoft\Launcher",
                r"SOFTWARE\WOW6432Node\Ubisoft\Launcher",
            ] {
                if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(key)
                    && let Ok(path) = key.get_value::<String, _>("InstallDir")
                {
                    for exe in id.executables() {
                        candidates.push(PathBuf::from(&path).join(exe));
                    }
                }
            }
        }
        for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            for exe in id.executables() {
                if let Ok(key) = RegKey::predef(hive).open_subkey(format!(
                    r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{exe}"
                )) && let Ok(path) = key.get_value::<String, _>("")
                {
                    candidates.push(PathBuf::from(path.trim_matches('"')));
                }
            }
            for uninstall in [
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ] {
                let Ok(parent) = RegKey::predef(hive).open_subkey(uninstall) else {
                    continue;
                };
                for name in parent.enum_keys().flatten() {
                    let Ok(key) = parent.open_subkey(name) else {
                        continue;
                    };
                    let Ok(display_name) = key.get_value::<String, _>("DisplayName") else {
                        continue;
                    };
                    if !id.matches_display_name(&display_name) {
                        continue;
                    }
                    if let Ok(location) = key.get_value::<String, _>("InstallLocation") {
                        let root = PathBuf::from(location.trim_matches('"'));
                        for exe in id.executables() {
                            candidates.push(root.join(exe));
                        }
                        if id == LauncherId::Epic {
                            for suffix in [
                                r"Portal\Binaries\Win64\EpicGamesLauncher.exe",
                                r"Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
                            ] {
                                candidates.push(root.join(suffix));
                            }
                        }
                    }
                    if let Ok(display_icon) = key.get_value::<String, _>("DisplayIcon") {
                        let path = display_icon
                            .rsplit_once(',')
                            .map_or(display_icon.as_str(), |(path, _)| path)
                            .trim()
                            .trim_matches('"');
                        let path = PathBuf::from(path);
                        if path.file_name().is_some_and(|file| {
                            id.executables()
                                .iter()
                                .any(|exe| file.to_string_lossy().eq_ignore_ascii_case(exe))
                        }) {
                            candidates.push(path);
                        }
                    }
                }
            }
        }
        candidates
            .into_iter()
            .find(|p| validate_executable(p).is_ok())
            .or_else(|| {
                common_installation(
                    id,
                    [
                        "ProgramFiles",
                        "ProgramFiles(x86)",
                        "ProgramW6432",
                        "LOCALAPPDATA",
                    ]
                    .into_iter()
                    .filter_map(env::var_os)
                    .map(PathBuf::from),
                )
            })
            .map(LaunchTarget::Executable)
    }
    pub fn set_executable(&mut self, id: LauncherId, path: PathBuf) -> Result<()> {
        let path = validate_executable(&path)?;
        let mut settings = self.settings.clone();
        settings.executable_overrides.insert(id, path);
        self.save_settings(settings)
    }
    pub fn is_enabled(&self, id: LauncherId) -> bool {
        !self.settings.disabled_launchers.contains(&id)
    }
    pub fn set_enabled(&mut self, id: LauncherId, enabled: bool) -> Result<()> {
        let mut settings = self.settings.clone();
        if enabled {
            settings.disabled_launchers.remove(&id);
        } else {
            settings.disabled_launchers.insert(id);
        }
        self.save_settings(settings)
    }
    fn save_settings(&mut self, settings: Settings) -> Result<()> {
        let parent = self
            .settings_path
            .parent()
            .context("Settings path has no parent")?;
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&serde_json::to_vec_pretty(&settings)?)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&self.settings_path)
            .context("Could not save Portal settings")?;
        self.settings = settings;
        Ok(())
    }
    pub fn launch(&self, id: LauncherId) -> Result<()> {
        if !self.is_enabled(id) {
            bail!("{} integration is disabled in Settings", id.name());
        }
        match self
            .discover(id)
            .context("Launcher not found. Select its executable using the folder button.")?
        {
            LaunchTarget::Executable(path) => {
                let path = validate_executable(&path)?;
                Command::new(&path)
                    .env_remove("ELECTRON_RUN_AS_NODE")
                    .current_dir(path.parent().context("Executable has no parent")?)
                    .spawn()
                    .context("Could not launch this platform")?;
            }
            LaunchTarget::WindowsApp(app_id) => {
                let _apartment = WindowsApartment::initialize()?;
                let activation: IApplicationActivationManager = unsafe {
                    CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_LOCAL_SERVER)?
                };
                let app_id: Vec<u16> = app_id.encode_utf16().chain(Some(0)).collect();
                unsafe {
                    activation.ActivateApplication(
                        PCWSTR(app_id.as_ptr()),
                        PCWSTR::null(),
                        AO_NONE,
                    )?;
                }
            }
            LaunchTarget::XboxProtocol(scheme) => {
                if !protocol_registered(scheme) {
                    bail!("Xbox is not registered on this PC")
                }
                let uri: Vec<u16> = format!("{scheme}:").encode_utf16().chain(Some(0)).collect();
                let result = unsafe {
                    ShellExecuteW(
                        None,
                        w!("open"),
                        PCWSTR(uri.as_ptr()),
                        PCWSTR::null(),
                        PCWSTR::null(),
                        SW_SHOWNORMAL,
                    )
                };
                if result.0 as isize <= 32 {
                    bail!("Windows could not open Xbox")
                }
            }
        }
        Ok(())
    }
}

fn protocol_registered(scheme: &str) -> bool {
    RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey(scheme)
        .is_ok_and(|key| key.get_raw_value("URL Protocol").is_ok())
}

struct WindowsApartment;
impl WindowsApartment {
    fn initialize() -> windows::core::Result<Self> {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)?;
        }
        Ok(Self)
    }
}
impl Drop for WindowsApartment {
    fn drop(&mut self) {
        unsafe {
            RoUninitialize();
        }
    }
}

fn discover_xbox() -> Option<LaunchTarget> {
    discover_xbox_app()
        .map(LaunchTarget::WindowsApp)
        .or_else(|| {
            ["xbox", "ms-xbox"]
                .into_iter()
                .find(|scheme| protocol_registered(scheme))
                .map(LaunchTarget::XboxProtocol)
        })
}

fn discover_xbox_app() -> Option<String> {
    let _apartment = WindowsApartment::initialize().ok()?;
    let family = "Microsoft.GamingApp_8wekyb3d8bbwe";
    let packages = PackageManager::new()
        .ok()?
        .FindPackagesByUserSecurityIdPackageFamilyName(&HSTRING::new(), &HSTRING::from(family))
        .ok()?;
    for package in packages {
        let entries = package.GetAppListEntriesAsync().ok()?.get().ok()?;
        for entry in entries {
            let app_id = entry.AppUserModelId().ok()?.to_string();
            if app_id.starts_with(&format!("{family}!")) {
                return Some(app_id);
            }
        }
    }
    None
}

fn validate_executable(path: &Path) -> Result<PathBuf> {
    if !path.is_file()
        || !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
    {
        bail!("Select an existing Windows executable (.exe).")
    }
    path.canonicalize()
        .context("Could not resolve executable path")
}

fn common_installation(
    id: LauncherId,
    roots: impl IntoIterator<Item = PathBuf>,
) -> Option<PathBuf> {
    roots
        .into_iter()
        .flat_map(|root| {
            id.relative_paths()
                .iter()
                .map(move |suffix| root.join(suffix))
        })
        .find(|path| validate_executable(path).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovers_the_three_requested_launcher_installations() -> Result<()> {
        let directory = tempfile::tempdir()?;
        for (id, path) in [
            (LauncherId::Amazon, r"Amazon Games\App\Amazon Games.exe"),
            (
                LauncherId::Rockstar,
                r"Rockstar Games\Launcher\LauncherPatcher.exe",
            ),
            (LauncherId::BattleNet, r"Battle.net\Battle.net Launcher.exe"),
        ] {
            let executable = directory.path().join(path);
            fs::create_dir_all(executable.parent().unwrap())?;
            fs::write(&executable, b"Test Launcher fixture")?;
            assert_eq!(
                common_installation(id, [directory.path().to_path_buf()]),
                Some(executable)
            );
        }
        Ok(())
    }
    #[test]
    fn override_round_trips_and_rejects_non_executables() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let executable = directory.path().join("Test Launcher & quoted.exe");
        fs::write(&executable, b"synthetic fixture")?;
        let settings = directory.path().join("settings.json");
        let mut catalog = Catalog::load_at(settings.clone())?;
        catalog.set_executable(LauncherId::Epic, executable.clone())?;
        assert!(
            matches!(Catalog::load_at(settings.clone())?.discover(LauncherId::Epic), Some(LaunchTarget::Executable(p)) if p == executable.canonicalize()?)
        );
        let before = fs::read(&settings)?;
        assert!(
            catalog
                .set_executable(LauncherId::Epic, settings.clone())
                .is_err()
        );
        assert_eq!(fs::read(settings)?, before);
        Ok(())
    }
    #[test]
    fn integrations_default_on_and_persist_without_losing_paths() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let settings = directory.path().join("settings.json");
        fs::write(&settings, br#"{"executable_overrides":{}}"#)?;
        let mut catalog = Catalog::load_at(settings.clone())?;
        assert!(LauncherId::ALL.into_iter().all(|id| catalog.is_enabled(id)));
        let executable = directory.path().join("Test Launcher.exe");
        fs::write(&executable, b"synthetic fixture")?;
        catalog.set_executable(LauncherId::Epic, executable.clone())?;
        catalog.set_enabled(LauncherId::Epic, false)?;
        let mut restored = Catalog::load_at(settings)?;
        assert!(!restored.is_enabled(LauncherId::Epic));
        assert!(restored.scan().iter().all(|l| l.id != LauncherId::Epic));
        assert!(
            restored
                .launch(LauncherId::Epic)
                .unwrap_err()
                .to_string()
                .contains("disabled")
        );
        restored.set_enabled(LauncherId::Epic, true)?;
        assert!(
            matches!(Catalog::load_at(restored.settings_path)?.discover(LauncherId::Epic), Some(LaunchTarget::Executable(path)) if path == executable.canonicalize()?)
        );
        Ok(())
    }
    #[test]
    fn corrupt_settings_are_preserved() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let settings = directory.path().join("settings.json");
        fs::write(&settings, b"invalid json")?;
        assert!(Catalog::load_at(settings.clone()).is_err());
        assert_eq!(fs::read(settings)?, b"invalid json");
        Ok(())
    }
}

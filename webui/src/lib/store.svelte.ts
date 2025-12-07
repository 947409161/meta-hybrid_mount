import { API } from './api';
import { DEFAULT_CONFIG, DEFAULT_SEED } from './constants';
import { APP_VERSION } from './constants_gen';
import { Monet } from './theme';
import type { 
  AppConfig, 
  Module, 
  StorageStatus, 
  SystemInfo, 
  DeviceInfo, 
  ToastMessage, 
  LanguageOption,
  ModeStats
} from './types';

const localeModules = import.meta.glob('../locales/*.json', { eager: true });

export interface LogEntry {
  text: string;
  type: 'info' | 'warn' | 'error' | 'debug';
}

const createStore = () => {
  let theme = $state<'auto' | 'light' | 'dark'>('auto');
  let isSystemDark = $state(false);
  let lang = $state('en');
  let seed = $state<string | null>(DEFAULT_SEED);
  let loadedLocale = $state<any>(null);
  let toast = $state<ToastMessage>({ id: 'init', text: '', type: 'info', visible: false });

  const availableLanguages: LanguageOption[] = Object.entries(localeModules).map(([path, mod]: [string, any]) => {
    const match = path.match(/\/([^/]+)\.json$/);
    const code = match ? match[1] : 'en';
    const name = mod.default?.lang?.display || code.toUpperCase();
    return { code, name };
  }).sort((a, b) => {
    if (a.code === 'en') return -1;
    if (b.code === 'en') return 1;
    return a.code.localeCompare(b.code);
  });

  let config = $state<AppConfig>({ ...DEFAULT_CONFIG });
  let modules = $state<Module[]>([]);
  let logs = $state<LogEntry[]>([]);
  
  let device = $state<DeviceInfo>({ model: 'Loading...', android: '-', kernel: '-', selinux: '-' });
  let version = $state(APP_VERSION);
  let storage = $state<StorageStatus>({ used: '-', size: '-', percent: '0%', type: null });
  let systemInfo = $state<SystemInfo>({ kernel: '-', selinux: '-', mountBase: '-', activeMounts: [] });
  let activePartitions = $state<string[]>([]);

  let loadingConfig = $state(false);
  let savingConfig = $state(false);
  let loadingModules = $state(false);
  let savingModules = $state(false);
  let loadingLogs = $state(false);
  let loadingStatus = $state(false);

  function getFallbackLocale() {
    return {
        common: { appName: "Hybrid Mount", saving: "...", theme: "Theme", language: "Language", themeAuto: "Auto", themeLight: "Light", themeDark: "Dark" },
        lang: { display: "English" },
        tabs: { status: "Status", config: "Config", modules: "Modules", logs: "Logs", info: "Info" },
        status: { storageTitle: "Storage", storageDesc: "Usage", moduleTitle: "Modules", moduleActive: "Active Modules", modeStats: "Stats", modeAuto: "OverlayFS", modeMagic: "Magic Mount", sysInfoTitle: "System Info", kernel: "Kernel", selinux: "SELinux", mountBase: "Mount Base", activePartitions: "Active Partitions" },
        config: { title: "Config", verboseLabel: "Verbose", verboseOff: "Off", verboseOn: "On", forceExt4: "Force Ext4", enableNuke: "Enable Nuke", disableUmount: "Disable Umount", moduleDir: "Module Dir", tempDir: "Temp Dir", mountSource: "Mount Source", logFile: "Log File", partitions: "Partitions", autoPlaceholder: "Auto", reload: "Reload", save: "Save", reset: "Reset", invalidPath: "Invalid path", loadSuccess: "Config Loaded", loadError: "Load Error", loadDefault: "Using Default", saveSuccess: "Saved", saveFailed: "Save Failed" },
        modules: { title: "Modules", desc: "Modules strictly managed by Magic Mount.", scanning: "Scanning...", reload: "Refresh", save: "Save", empty: "No modules", scanError: "Scan Failed", saveSuccess: "Saved", saveFailed: "Failed", searchPlaceholder: "Search", filterLabel: "Filter", filterAll: "All", modeAuto: "OverlayFS", modeMagic: "Magic Mount" },
        logs: { title: "Logs", loading: "Loading...", refresh: "Refresh", empty: "Empty", copy: "Copy", export: "Export", copySuccess: "Copied", copyFail: "Failed", exportFail: "Export Failed", searchPlaceholder: "Search", filterLabel: "Level", levels: { all: "All", info: "Info", warn: "Warn", error: "Error" }, select: "File", current: "Current", old: "Old", readFailed: "Read Failed", readException: "Exception" },
        info: { title: "About", projectLink: "Repository", donate: "Donate", contributors: "Contributors", loading: "Loading...", loadFail: "Failed to load", noBio: "No bio available" }
    };
  }

  const L = $derived(loadedLocale || getFallbackLocale());

  const modeStats = $derived.by<ModeStats>(() => {
    let auto = 0;
    let magic = 0;
    modules.forEach(m => {
      if (m.mode === 'magic') magic++;
      else auto++;
    });
    return { auto, magic };
  });

  function showToast(msg: string, type: 'info' | 'success' | 'error' = 'info') {
    const id = Date.now().toString();
    toast = { id, text: msg, type, visible: true };
    setTimeout(() => { 
        if (toast.id === id) toast.visible = false; 
    }, 3000);
  }

  function applyTheme() {
    const isDark = theme === 'auto' ? isSystemDark : theme === 'dark';
    const attr = isDark ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', attr);
    Monet.apply(seed, isDark);
  }

  function setTheme(newTheme: 'auto' | 'light' | 'dark') {
    theme = newTheme;
    localStorage.setItem('hm-theme', newTheme);
    applyTheme();
  }

  async function setLang(code: string) {
    const path = `../locales/${code}.json`;
    if ((localeModules as any)[path]) {
      try {
        const mod: any = (localeModules as any)[path];
        loadedLocale = mod.default; 
        lang = code;
        localStorage.setItem('hm-lang', code);
      } catch (e) {
        console.error(`Failed to load locale: ${code}`, e);
        if (code !== 'en') await setLang('en');
      }
    }
  }

  async function init() {
    const savedLang = localStorage.getItem('hm-lang') || 'en';
    await setLang(savedLang);
    
    const savedTheme = localStorage.getItem('hm-theme');
    if (savedTheme === 'light' || savedTheme === 'dark' || savedTheme === 'auto') {
        theme = savedTheme;
    } else {
        theme = 'auto';
    }
    
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    isSystemDark = mediaQuery.matches;
    
    mediaQuery.addEventListener('change', (e) => {
      isSystemDark = e.matches;
      if (theme === 'auto') {
        applyTheme();
      }
    });
    
    const sysColor = await API.fetchSystemColor();
    if (sysColor) {
      seed = sysColor;
    }
    applyTheme();

    await loadConfig();
  }

  async function loadConfig() {
    loadingConfig = true;
    try {
      config = await API.loadConfig();
      if (loadedLocale) showToast(L.config.loadSuccess);
    } catch (e) {
      if (loadedLocale) showToast(L.config.loadError, 'error');
    }
    loadingConfig = false;
  }

  async function saveConfig() {
    savingConfig = true;
    try {
      await API.saveConfig(config);
      showToast(L.config.saveSuccess);
    } catch (e) {
      showToast(L.config.saveFailed, 'error');
    }
    savingConfig = false;
  }

  async function loadModules() {
    loadingModules = true;
    try {
      modules = await API.scanModules(config.moduledir);
    } catch (e) {
      showToast(L.modules.scanError, 'error');
    }
    loadingModules = false;
  }

  async function saveModules() {
    savingModules = true;
    try {
      await API.saveModules(modules);
      showToast(L.modules.saveSuccess);
    } catch (e) {
      showToast(L.modules.saveFailed, 'error');
    }
    savingModules = false;
  }

  async function loadLogs(silent = false) {
    if (!silent) loadingLogs = true;
    try {
      const raw = await API.readLogs();
      if (!raw) {
        logs = [];
      } else {
        logs = raw.split('\n').map(line => {
          let type: LogEntry['type'] = 'debug';
          if (line.includes('ERROR') || line.includes('[E]')) type = 'error';
          else if (line.includes('WARN') || line.includes('[W]')) type = 'warn';
          else if (line.includes('INFO') || line.includes('[I]')) type = 'info';
          return { text: line, type };
        });
      }
    } catch (e: any) {
      logs = [{ text: `Error loading logs: ${e.message}`, type: 'error' }];
      if (!silent) showToast(L.logs.readFailed, 'error');
    }
    loadingLogs = false;
  }

  async function loadStatus() {
    loadingStatus = true;
    try {
      device = await API.getDeviceStatus();
      version = await API.getVersion();
      if (version) {
        version = version;
      }
      storage = await API.getStorageUsage();
      systemInfo = await API.getSystemInfo();
      activePartitions = systemInfo.activeMounts || [];
      
      if (modules.length === 0) {
        await loadModules();
      }
    } catch (e) {}
    loadingStatus = false;
  }

  return {
    get theme() { return theme; },
    get isSystemDark() { return isSystemDark; },
    get lang() { return lang; },
    get seed() { return seed; },
    get availableLanguages() { return availableLanguages; },
    get L() { return L; },
    get toast() { return toast; },
    get toasts() { return toast.visible ? [toast] : []; },
    showToast,
    setTheme,
    setLang,
    init,

    get config() { return config; },
    set config(v) { config = v; },
    loadConfig,
    saveConfig,

    get modules() { return modules; },
    set modules(v) { modules = v; },
    get modeStats() { return modeStats; },
    loadModules,
    saveModules,

    get logs() { return logs; },
    loadLogs,

    get device() { return device; },
    get version() { return version; },
    get storage() { return storage; },
    get systemInfo() { return systemInfo; },
    get activePartitions() { return activePartitions; },
    loadStatus,

    get loading() {
      return {
        config: loadingConfig,
        modules: loadingModules,
        logs: loadingLogs,
        status: loadingStatus
      };
    },
    get saving() {
      return {
        config: savingConfig,
        modules: savingModules
      };
    }
  };
};

export const store = createStore();
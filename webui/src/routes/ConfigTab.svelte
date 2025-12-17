<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import ChipInput from '../components/ChipInput.svelte';
  import BottomActions from '../components/BottomActions.svelte';
  import { slide } from 'svelte/transition';
  import './ConfigTab.css';
  import '@material/web/textfield/outlined-text-field.js';
  import '@material/web/button/filled-button.js';
  import '@material/web/iconbutton/filled-tonal-icon-button.js';
  import '@material/web/iconbutton/icon-button.js';
  import '@material/web/icon/icon.js';
  import '@material/web/ripple/ripple.js';
  import '@material/web/dialog/dialog.js';
  import '@material/web/button/text-button.js';

  let initialConfigStr = $state('');
  let showResetConfirm = $state(false);

  const isValidPath = (p: string) => !p || (p.startsWith('/') && p.length > 1);
  let invalidModuleDir = $derived(!isValidPath(store.config.moduledir));
  let invalidTempDir = $derived(store.config.tempdir && !isValidPath(store.config.tempdir));
  let isDirty = $derived.by(() => {
    if (!initialConfigStr) return false;
    return JSON.stringify(store.config) !== initialConfigStr;
  });

  $effect(() => {
    if (!store.loading.config && store.config) {
      if (!initialConfigStr || initialConfigStr === JSON.stringify(store.config)) {
        initialConfigStr = JSON.stringify(store.config);
      }
    }
  });

  $effect(() => {
    if (store.systemInfo?.zygisksuEnforce && store.systemInfo.zygisksuEnforce !== '0' && !store.config.allow_umount_coexistence) {
        if (!store.config.disable_umount) {
            store.config.disable_umount = true;
        }
    }
  });

  function save() {
    if (invalidModuleDir || invalidTempDir) {
      store.showToast(store.L.config.invalidPath, "error");
      return;
    }
    store.saveConfig().then(() => {
        initialConfigStr = JSON.stringify(store.config);
    });
  }

  function reload() {
    store.loadConfig().then(() => {
        initialConfigStr = JSON.stringify(store.config);
    });
  }
  
  function reset() {
    showResetConfirm = false;
    store.resetConfig().then(() => {
        initialConfigStr = JSON.stringify(store.config);
    });
  }

  function resetTempDir() {
    store.config.tempdir = "";
  }

  function toggle(key: keyof typeof store.config) {
    if (key === 'disable_umount') {
       if (store.systemInfo?.zygisksuEnforce && store.systemInfo.zygisksuEnforce !== '0' && !store.config.allow_umount_coexistence) {
          store.showToast(store.L.config?.coexistenceRequired || "Coexistence required", "error");
          return;
       }
    }
    if (typeof store.config[key] === 'boolean') {
      (store.config as any)[key] = !store.config[key];
    }
  }

  function handleInput(e: Event, key: keyof typeof store.config) {
    const target = e.target as HTMLInputElement;
    (store.config as any)[key] = target.value;
  }
  
  const REPLAY_ICON = "M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z";
  const ICON_STEALTH = "M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z";
  const ICON_BUG = "M20 8h-2.81c-.45-.78-1.07-1.45-1.82-1.96L17 4.41 15.59 3l-2.17 2.17C12.96 5.06 12.49 5 12 5c-.49 0-.96.06-1.41.17L8.41 3 7 4.41l1.62 1.63C7.88 6.55 7.26 7.22 6.81 8H4v2h2.09c-.05.33-.09.66-.09 1v1H4v2h2v1c0 .34.04.67.09 1H4v2h2.81c1.04 1.79 2.97 3 5.19 3s4.15-1.21 5.19-3H20v-2h-2.09c.05-.33.09-.66.09-1v-1h2v-2h-2v-1c0-.34-.04-.67-.09-1H20V8zm-6 8h-4v-2h4v2zm0-4h-4v-2h4v2z";
</script>

<md-dialog 
  open={showResetConfirm} 
  onclose={() => showResetConfirm = false}
  style="--md-dialog-scrim-color: transparent; --md-sys-color-scrim: transparent;"
>
  <div slot="headline">{store.L.config?.resetConfigTitle ?? 'Reset Configuration?'}</div>
  <div slot="content">
    {store.L.config?.resetConfigConfirm ?? 'This will reset all backend settings to defaults. Continue?'}
  </div>
  <div slot="actions">
    <md-text-button 
      onclick={() => showResetConfirm = false}
      role="button"
      tabindex="0"
      onkeydown={() => {}}
    >
      {store.L.common?.cancel ?? 'Cancel'}
    </md-text-button>
    <md-text-button 
      onclick={reset}
      role="button"
      tabindex="0"
      onkeydown={() => {}}
    >
      {store.L.config?.resetConfig ?? 'Reset Config'}
    </md-text-button>
  </div>
</md-dialog>

<div class="config-container">
  <section class="config-group">
    <div class="config-card">
      <div class="card-header">
        <div class="card-icon">
          <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.storage} /></svg></md-icon>
        </div>
        <div class="card-text">
          <span class="card-title">{store.L.status?.storageTitle ?? 'Storage'}</span>
          <span class="card-desc">Configure paths</span>
        </div>
      </div>
      
      <div class="input-stack">
        <md-outlined-text-field 
          label={store.L.config.moduleDir} 
          value={store.config.moduledir}
          oninput={(e) => handleInput(e, 'moduledir')}
          error={invalidModuleDir}
          class="full-width-field"
        >
          <md-icon slot="leading-icon"><svg viewBox="0 0 24 24"><path d={ICONS.modules} /></svg></md-icon>
        </md-outlined-text-field>

        <md-outlined-text-field 
          label={store.L.config.tempDir} 
          value={store.config.tempdir}
          oninput={(e) => handleInput(e, 'tempdir')}
          placeholder={store.L.config.autoPlaceholder}
          error={invalidTempDir}
          class="full-width-field"
        >
          <md-icon slot="leading-icon"><svg viewBox="0 0 24 24"><path d={ICONS.timer} /></svg></md-icon>
          {#if store.config.tempdir}
            <md-icon-button 
                slot="trailing-icon" 
                onclick={resetTempDir}
                role="button"
                tabindex="0"
                onkeydown={() => {}}
            >
              <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.close} /></svg></md-icon>
            </md-icon-button>
          {/if}
        </md-outlined-text-field>
      </div>
    </div>
  </section>

  <section class="config-group">
    <div class="config-card">
      <div class="card-header">
        <div class="card-icon">
          <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.storage} /></svg></md-icon>
        </div>
        <div class="card-text">
          <span class="card-title">{store.L.config.partitions}</span>
          <span class="card-desc">Add partitions to mount</span>
        </div>
      </div>
      <div class="p-input">
        <ChipInput bind:values={store.config.partitions} placeholder="e.g. product, system_ext..." />
      </div>
    </div>
  </section>

  <section class="config-group">
    <div class="options-grid">
      <div class="option-tile static-input">
        <div class="tile-top">
          <div class="tile-icon neutral">
            <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.ksu} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.mountSource}</span>
          <input class="tile-input-overlay" type="text" bind:value={store.config.mountsource} />
        </div>
      </div>

      <button 
        class="option-tile clickable secondary" 
        class:active={store.config.force_ext4} 
        onclick={() => toggle('force_ext4')}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
            <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.save} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.forceExt4}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable error" 
        class:active={store.config.enable_nuke} 
        onclick={() => toggle('enable_nuke')}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
            <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.cat_paw} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.enableNuke}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable tertiary" 
        class:active={store.config.disable_umount} 
        onclick={() => toggle('disable_umount')}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
            <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.anchor} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.disableUmount}</span>
        </div>
      </button>

      {#if store.systemInfo?.zygisksuEnforce && store.systemInfo.zygisksuEnforce !== '0'}
        <button 
          class="option-tile clickable error" 
          class:active={store.config.allow_umount_coexistence} 
          onclick={() => toggle('allow_umount_coexistence')}
          transition:slide
        >
          <md-ripple></md-ripple>
          <div class="tile-top">
            <div class="tile-icon">
              <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.shield} /></svg></md-icon>
            </div>
          </div>
          <div class="tile-bottom">
              <span class="tile-label">{store.L.config?.allowUmountCoexistence || 'Allow Coexistence'}</span>
          </div>
        </button>
      {/if}

      <button 
        class="option-tile clickable error" 
        class:active={store.config.hymofs_stealth} 
        onclick={() => toggle('hymofs_stealth')}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
             <md-icon><svg viewBox="0 0 24 24"><path d={ICON_STEALTH} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config?.hymoStealth || 'Hymo Stealth'}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable secondary" 
        class:active={store.config.hymofs_debug} 
        onclick={() => toggle('hymofs_debug')}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
             <md-icon><svg viewBox="0 0 24 24"><path d={ICON_BUG} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config?.hymoDebug || 'Hymo Debug'}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable primary" 
        class:active={store.config.verbose} 
        onclick={() => toggle('verbose')}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
             <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.description} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.verboseLabel}</span>
        </div>
      </button>

      {#if store.config.verbose}
        <button 
          class="option-tile clickable secondary" 
          class:active={store.config.dry_run} 
          onclick={() => toggle('dry_run')}
          transition:slide
        >
          <md-ripple></md-ripple>
          <div class="tile-top">
            <div class="tile-icon">
              <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.ghost} /></svg></md-icon>
             </div>
          </div>
          <div class="tile-bottom">
            <span class="tile-label">{store.L.config.dryRun}</span>
          </div>
        </button>
      {/if}
    </div>
  </section>

  <section class="config-group">
    <div class="webui-label">
        {store.L.config?.webui || 'WebUI'}
    </div>
    <div class="options-grid">
      <button 
        class="option-tile clickable secondary" 
        class:active={store.fixBottomNav} 
        onclick={store.toggleBottomNavFix}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
            <md-icon><svg viewBox="0 0 24 24"><path d="M21 5v14H3V5h18zm0-2H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM8 17h5v-6H8v6zm0-8h5V7H8v2zM6 17h2V7H6v10zm12-6h-2v6h2v-6zm0-4h-2v2h2V7z" /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config?.fixBottomNav || 'Fix Bottom Nav'}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable error" 
        onclick={() => showResetConfirm = true}
        disabled={store.saving.config}
      >
        <md-ripple></md-ripple>
        <div class="tile-top">
          <div class="tile-icon">
            <md-icon><svg viewBox="0 0 24 24"><path d={REPLAY_ICON} /></svg></md-icon>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config?.resetConfig || 'Reset Config'}</span>
        </div>
      </button>
    </div>
  </section>
</div>

<BottomActions>
  <md-filled-tonal-icon-button 
    onclick={reload}
    disabled={store.loading.config}
    title={store.L.config.reload}
    role="button"
    tabindex="0"
    onkeydown={() => {}}
  >
    <md-icon><svg viewBox="0 0 24 24"><path d={ICONS.refresh} /></svg></md-icon>
  </md-filled-tonal-icon-button>
  
  <div class="spacer"></div>

  <md-filled-button 
    onclick={save} 
    disabled={store.saving.config || !isDirty}
    role="button"
    tabindex="0"
    onkeydown={() => {}}
  >
    <md-icon slot="icon"><svg viewBox="0 0 24 24"><path d={ICONS.save} /></svg></md-icon>
    {store.saving.config ? store.L.common.saving : store.L.config.save}
  </md-filled-button>
</BottomActions>
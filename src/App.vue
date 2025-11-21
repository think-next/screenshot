<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";

const showOverlayLocal = ref(false); // kept for dev fallback if needed
const selectionInfo = ref<{x:number,y:number,w:number,h:number}|null>(null);

let selectionUnlisten: (() => void) | null = null;

async function startOverlayWindow() {
    try {
      // First check we're running inside Tauri
      // prefer a quick runtime check for Tauri environment
      let isTauri = false;
      try {
        isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI__;
      } catch (_) {
        isTauri = false;
      }
      if (!isTauri) {
        console.warn('not running inside Tauri runtime; falling back to in-app overlay');
        showOverlayLocal.value = true;
        return;
      }

      // dynamic import to avoid static typing/export differences across @tauri-apps/api versions
      const tauriWindow: any = await import('@tauri-apps/api/window');
      const WebviewWindow = tauriWindow.WebviewWindow || tauriWindow.Window || tauriWindow.createWindow;
      const getCurrent = tauriWindow.getCurrent;
      const appWindow = tauriWindow.appWindow;

      // hide the current main window (try getCurrent -> appWindow)
      try {
        if (getCurrent) {
          const current = getCurrent();
          await current.hide();
        } else if (appWindow && appWindow.hide) {
          await appWindow.hide();
        }
      } catch (hideErr) {
        console.warn('failed to hide main window', hideErr);
      }

      // create overlay window if constructor available
      if (typeof WebviewWindow === 'function') {
        new WebviewWindow('overlay', {
          url: 'overlay.html',
          title: 'Overlay',
          transparent: true,
          decorations: false,
          alwaysOnTop: true,
          fullscreen: true,
          visible: true,
        });
      } else {
        console.error('WebviewWindow constructor not available on @tauri-apps/api/window', tauriWindow);
        showOverlayLocal.value = true;
      }
    } catch (e) {
    // fallback: show local overlay inside the app (useful during web dev)
    console.warn('failed to create overlay window, falling back to in-app overlay', e);
    showOverlayLocal.value = true;
  }
}

function hideOverlayLocal() {
  showOverlayLocal.value = false;
}

async function handleSelectionEvent(payload: any) {
  console.log('screenshot-selection received', payload);
  // show the selection briefly in the UI for verification
  if (payload && typeof payload.x === 'number') {
    selectionInfo.value = { x: payload.x, y: payload.y, w: payload.w, h: payload.h };
  }
  // restore main window if possible
  try {
    const tauriWindow: any = await import('@tauri-apps/api/window');
    const WebviewWindow = tauriWindow.WebviewWindow;
    if (WebviewWindow && WebviewWindow.getByLabel) {
      const main = WebviewWindow.getByLabel('main');
      if (main && main.show) await main.show();
    }
  } catch (e) {
    console.warn('restore main window failed', e);
  }
}

onMounted(async () => {
  try {
    const evt: any = await import('@tauri-apps/api/event');
    // listen for selection events emitted by overlay
    selectionUnlisten = await evt.listen('screenshot-selection', (e: any) => {
      handleSelectionEvent(e.payload);
    });
  } catch (e) {
    console.warn('event listen failed', e);
  }
});

onBeforeUnmount(() => {
  if (selectionUnlisten) selectionUnlisten();
});
</script>

<template>
  <div>
    <div class="toolbar">
      <div class="toolbar-inner">
        <button class="screenshot-btn" @click="startOverlayWindow">截屏</button>
      </div>
    </div>

    <main class="container">
      <h1>轻量截屏工具</h1>
      <p class="subtitle">点击上方按钮开始截屏（演示遮罩）</p>
      <div v-if="selectionInfo" class="selection-info">
        选区: x={{selectionInfo.x}} y={{selectionInfo.y}} w={{selectionInfo.w}} h={{selectionInfo.h}}
      </div>
    </main>

    <!-- local fallback overlay when WebviewWindow isn't available (e.g., non-tauri web dev) -->
    <div v-if="showOverlayLocal" class="overlay" @click.self="hideOverlayLocal">
      <div class="overlay-center">hello</div>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  position: fixed;
  top: 14px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
}

.toolbar-inner {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 12px;
  box-shadow: 0 6px 18px rgba(16, 24, 40, 0.12);
  padding: 8px 12px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.screenshot-btn {
  background: #396cd8;
  color: white;
  border: none;
  padding: 8px 12px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
}

.screenshot-btn:active {
  transform: translateY(1px);
}

.container {
  margin: 0;
  padding-top: 18vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.subtitle {
  color: #666;
  margin-top: 6px;
}

.selection-info {
  margin-top: 12px;
  font-size: 13px;
  color: #333;
  background: #f5f7ff;
  display: inline-block;
  padding: 6px 10px;
  border-radius: 6px;
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.14);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9998;
}

.overlay-center {
  pointer-events: none;
  color: white;
  font-size: 48px;
  font-weight: 700;
  background: rgba(0,0,0,0.3);
  padding: 20px 32px;
  border-radius: 8px;
}

@media (prefers-color-scheme: dark) {
  .toolbar-inner {
    background: rgba(20, 20, 20, 0.85);
    color: #fff;
  }
  .screenshot-btn {
    background: #2b6cb0;
  }
}

</style>
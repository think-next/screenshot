<script setup lang="ts">
import { ref } from "vue";

const showOverlayLocal = ref(false); // kept for dev fallback if needed

async function startOverlayWindow() {
    try {
      // dynamic import to avoid static typing/export differences across @tauri-apps/api versions
      const tauriWindow: any = await import('@tauri-apps/api/window');
      const WebviewWindow = tauriWindow.WebviewWindow;
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

      // create overlay window
      new WebviewWindow('overlay', {
        url: 'overlay.html',
        title: 'Overlay',
        transparent: true,
        decorations: false,
        alwaysOnTop: true,
        fullscreen: true,
        visible: true,
      });
    } catch (e) {
    // fallback: show local overlay inside the app (useful during web dev)
    console.warn('failed to create overlay window, falling back to in-app overlay', e);
    showOverlayLocal.value = true;
  }
}

function hideOverlayLocal() {
  showOverlayLocal.value = false;
}
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
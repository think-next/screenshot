<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';

const showOverlayLocal = ref(false); // kept for dev fallback if needed
const selectionInfo = ref<{x:number,y:number,w:number,h:number}|null>(null);
const screenshotData = ref<string|null>(null);

let selectionUnlisten: (() => void) | null = null;

async function startOverlayWindow() {
  try {
    console.log('Starting screenshot process...');
    
    // 获取当前窗口实例
    const currentWindow = getCurrentWindow();
    console.log('Got current window object');

    // 隐藏当前窗口
    try {
      if (currentWindow && typeof currentWindow.hide === 'function') {
        console.log('Hiding window using currentWindow.hide()');
        await currentWindow.hide();
        console.log('Window hidden successfully');
      } else {
        console.warn('Current window does not have hide method');
        console.warn('Warning: Unable to hide window, proceeding with screenshot anyway');
      }
    } catch (hideErr) {
      console.error('Failed to hide main window:', hideErr);
      console.warn('窗口隐藏失败，但仍将继续截图流程');
    }

    // 等待一段时间确保窗口已隐藏
    console.log('Waiting for window to hide...');
    await new Promise(resolve => setTimeout(resolve, 500));

    // 调用截图功能
    try {
      console.log('Attempting to invoke capture_screen command');
      const tauriApi: any = await import('@tauri-apps/api/core');
      const imageData = await tauriApi.invoke('capture_screen');
      console.log('Screenshot command invoked successfully, received data length:', imageData.length);
      
      // 将截图数据保存到响应式变量中，设置为背景图
      screenshotData.value = `data:image/png;base64,${imageData}`;
      console.log('Screenshot captured and set as background');
    } catch (e) {
      console.error('Failed to capture screen:', e);
    }

    // 显示当前窗口
    try {
      console.log('Showing window again...');
      if (currentWindow && typeof currentWindow.show === 'function') {
        console.log('Showing window using currentWindow.show()');
        await currentWindow.show();
        console.log('Window shown successfully');
      } else {
        console.warn('Current window does not have show method');
      }
    } catch (showErr) {
      console.error('Failed to show main window:', showErr);
      console.warn('窗口显示失败');
    }
    
    console.log('Screenshot process completed.');
  } catch (e) {
    console.error('Error during screenshot process:', e);
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
  <div :class="{ 'with-background': screenshotData }" :style="screenshotData ? { backgroundImage: `url(${screenshotData})` } : {}">
    <div class="toolbar">
      <div class="toolbar-inner">
        <button class="screenshot-btn" @click="startOverlayWindow">截屏</button>
      </div>
    </div>

    <main class="container">
      <h1>轻量截屏工具</h1>
      <p class="subtitle">点击上方按钮开始截屏</p>
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

.with-background {
  background-size: cover;
  background-position: center;
  min-height: 100vh;
}

.with-background .toolbar-inner {
  background: rgba(255, 255, 255, 0.7);
}

.with-background .container {
  background: rgba(255, 255, 255, 0.7);
  padding: 20px;
  border-radius: 10px;
  backdrop-filter: blur(5px);
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
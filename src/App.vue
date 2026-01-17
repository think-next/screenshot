<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';

const showOverlayLocal = ref(false); // kept for dev fallback if needed
const selectionInfo = ref<{x:number,y:number,w:number,h:number}|null>(null);
const screenshotData = ref<string|null>(null);
const isFullscreenView = ref(false); // 是否处于全屏查看模式

let selectionUnlisten: (() => void) | null = null;

async function startOverlayWindow() {
  try {
    console.log('Starting screenshot process...');
    
    // 获取当前窗口实例
    const currentWindow = getCurrentWindow();
    console.log('Got current window object');

    if (currentWindow && typeof currentWindow.hide === 'function') {
        await currentWindow.hide();
        console.log('Window hide successfully');
      } else {
        console.warn('Current window does not have hide method');
      }

    // 等待一段时间确保窗口已全屏
    console.log('Waiting for window to go fullscreen...');
    await new Promise(resolve => setTimeout(resolve, 500));

    // 调用截图功能
    try {
      console.log('Attempting to invoke capture_screen command');
      const tauriApi: any = await import('@tauri-apps/api/core');
      const imageData = await tauriApi.invoke('capture_screen');
      console.log('Screenshot command invoked successfully, received data length:', imageData.length);
      
      // 将截图数据保存到响应式变量中，设置为背景图
      screenshotData.value = `data:image/jpeg;base64,${imageData}`;
      console.log('Screenshot captured and set as background');
    } catch (e) {
      console.error('Failed to capture screen:', e);
    }

    // 退出全屏并显示当前窗口
    try {
      if (currentWindow && typeof currentWindow.show === 'function') {
        console.log('Showing window using currentWindow.show()');
        await currentWindow.show();
        console.log('Window shown successfully');
      } else {
        console.warn('Current window does not have show method');
      }

      if (currentWindow && typeof currentWindow.setFullscreen === 'function') {
        console.log('Exiting fullscreen using currentWindow.setFullscreen()');
        await currentWindow.setFullscreen(true);
        console.log('Fullscreen exited successfully');
        // 进入全屏查看模式
        isFullscreenView.value = true;
      } else {
        console.warn('Current window does not have setFullscreen method');
      }
      
    } catch (showErr) {
      console.error('Failed to show main window or exit fullscreen:', showErr);
      console.warn('窗口显示或退出全屏失败');
    }
    
    console.log('Screenshot process completed.');
  } catch (e) {
    console.error('Error during screenshot process:', e);
  }
}

// 添加键盘事件监听器，用于ESC键退出全屏

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
    
    // 添加键盘事件监听器，用于ESC键退出全屏
    window.addEventListener('keydown', async (event) => {
      if (event.key === 'Escape') {
        const currentWindow = getCurrentWindow();
        try {
          // 退出全屏模式
          if (currentWindow && typeof currentWindow.setFullscreen === 'function') {
            await currentWindow.setFullscreen(false);
            console.log('Fullscreen exited via Escape key');
            // 退出全屏查看模式
            isFullscreenView.value = false;
            // 清除截图数据，恢复初始状态
            screenshotData.value = null;
          }
        } catch (err) {
          console.error('Failed to exit fullscreen:', err);
        }
      }
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
  <div :class="{ 'with-background': screenshotData, 'fullscreen-view': isFullscreenView }" :style="screenshotData ? { backgroundImage: `url(${screenshotData})` } : {}">
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

<style>
/* 重置html和body的默认间距 */
html, body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

#app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
}
</style>

<style scoped>
/* 重置根元素的默认间距 */
div {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

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
  width: 100vw;
  margin: 0;
  padding: 0;
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

/* 全屏查看模式：隐藏所有UI元素，只显示背景图 */
.fullscreen-view .toolbar {
  display: none;
}

.fullscreen-view .container {
  display: none;
}

.fullscreen-view .overlay {
  display: none;
}
</style>

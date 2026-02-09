<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import CrosshairComponent from './components/CrosshairComponent.vue';
import SelectionRectangleComponent from './components/SelectionRectangleComponent.vue';
import { useCrosshair } from './composables/useCrosshair';
import { useSelection } from './composables/useSelection';

const showOverlayLocal = ref(false); // kept for dev fallback if needed
const selectionInfo = ref<{x:number,y:number,w:number,h:number}|null>(null);
const screenshotData = ref<string|null>(null);
const isFullscreenView = ref(false); // 是否处于全屏查看模式

let selectionUnlisten: (() => void) | null = null;
const { position: mousePosition, startTracking, stopTracking } = useCrosshair();
const { isSelecting, selection, startSelection: startRegionSelection, stopSelection: stopRegionSelection, clearSelection } = useSelection();

// 获取设备像素比，用于在 Retina 显示器上正确映射坐标
const devicePixelRatio = window.devicePixelRatio || 1;
console.log('Device pixel ratio:', devicePixelRatio);

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
        console.log('Entering fullscreen mode');
        await currentWindow.setFullscreen(true);
        console.log('Fullscreen entered successfully');
        // 进入全屏查看模式
        isFullscreenView.value = true;
        
      // 启动十字线跟踪
      startTracking();
      
      // 启动区域选择
      startRegionSelection();
      } else {
        console.warn('Current window does not have setFullscreen method');
      }

      // 设置 macOS 全屏演示模式，自动隐藏菜单栏和 Dock
      try {
        const tauriApi: any = await import('@tauri-apps/api/core');
        await tauriApi.invoke('set_macos_presentation_mode', { fullscreen: true });
        console.log('macOS presentation mode set to fullscreen');
      } catch (presentErr) {
        console.warn('Failed to set macOS presentation mode:', presentErr);
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

// 处理确认截图事件
async function handleConfirmScreenshot() {
  console.log('handleConfirmScreenshot called');
  console.log('selection.value:', selection.value);
  console.log('isFullscreenView.value:', isFullscreenView.value);
  
  if (!selection.value || !isFullscreenView.value) {
    console.log('Early return: selection or isFullscreenView is false');
    return;
  }
  
  try {
    console.log('========================================');
    console.log('前端显示的CSS像素:', selection.value);
    console.log('devicePixelRatio:', devicePixelRatio);
    
    // 获取背景图容器的实际尺寸
    const bgElement = document.querySelector('#app') as HTMLElement;
    const bgWidth = bgElement?.offsetWidth || window.innerWidth;
    const bgHeight = bgElement?.offsetHeight || window.innerHeight;
    
    console.log(`背景图容器实际尺寸: ${bgWidth}x${bgHeight}`);
    
    // 前端的CSS像素坐标需要映射到屏幕的物理像素
    // 比例 = 屏幕宽度 / 容器宽度（通常 = devicePixelRatio）
    const scaleX = window.screen.width / bgWidth;
    const scaleY = window.screen.height / bgHeight;
    
    console.log(`坐标缩放比例: scaleX=${scaleX}, scaleY=${scaleY}`);
    
    // 应用缩放比例
    const x = Math.round(selection.value.startX * scaleX);
    const y = Math.round(selection.value.startY * scaleY);
    const width = Math.round(selection.value.width * scaleX);
    const height = Math.round(selection.value.height * scaleY);
    
    console.log('映射到屏幕物理像素:');
    console.log(`  x: ${x} (CSS: ${selection.value.startX} * ${scaleX})`);
    console.log(`  y: ${y} (CSS: ${selection.value.startY} * ${scaleY})`);
    console.log(`  width: ${width} (CSS: ${selection.value.width} * ${scaleX})`);
    console.log(`  height: ${height} (CSS: ${selection.value.height} * ${scaleY})`);
    console.log('========================================');
    
    console.log('About to invoke capture_and_copy_region...');
    // 调用后端命令捕获并复制到剪贴板
    const tauriApi: any = await import('@tauri-apps/api/core');
    console.log('tauriApi imported, invoking command...');
    const result = await tauriApi.invoke('capture_and_copy_region', {
      x: x,
      y: y,
      width: width,
      height: height
    });
    
    console.log('Screenshot copied to clipboard:', result);
    
    // 退出全屏模式
    const currentWindow = getCurrentWindow();
    if (currentWindow && typeof currentWindow.setFullscreen === 'function') {
      await currentWindow.setFullscreen(false);
      console.log('Fullscreen exited after screenshot');
      
      // 恢复 macOS 演示模式
      try {
        await tauriApi.invoke('set_macos_presentation_mode', { fullscreen: false });
        console.log('macOS presentation mode restored');
      } catch (presentErr) {
        console.warn('Failed to restore macOS presentation mode:', presentErr);
      }
      
      // 退出全屏查看模式
      isFullscreenView.value = false;
      screenshotData.value = null;
      stopTracking();
      stopRegionSelection();
      clearSelection();
    }
  } catch (err) {
    console.error('Failed to capture and copy screenshot:', err);
    alert('截图失败: ' + err);
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

            // 恢复 macOS 演示模式
            try {
              const tauriApi: any = await import('@tauri-apps/api/core');
              await tauriApi.invoke('set_macos_presentation_mode', { fullscreen: false });
              console.log('macOS presentation mode restored to normal');
            } catch (presentErr) {
              console.warn('Failed to restore macOS presentation mode:', presentErr);
            }

            // 退出全屏查看模式
            isFullscreenView.value = false;
            // 清除截图数据，恢复初始状态
            screenshotData.value = null;
            // 停止十字线跟踪
            stopTracking();
            
            // 停止区域选择
            stopRegionSelection();
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

    <!-- 十字线组件：在全屏查看模式下显示 -->
    <CrosshairComponent
      :visible="isFullscreenView"
      :position="mousePosition"
    />

    <!-- 选择矩形组件：在全屏查看模式下显示 -->
    <SelectionRectangleComponent
      :visible="isFullscreenView"
      :selection="selection"
      :show-dimensions="true"
      @confirm="handleConfirmScreenshot"
    />
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

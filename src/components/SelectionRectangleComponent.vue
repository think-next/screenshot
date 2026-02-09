<script setup lang="ts">
import { computed } from 'vue';
import type { Selection } from '../composables/useSelection';

interface Props {
  visible?: boolean
  selection?: Selection | null
  showDimensions?: boolean
}

interface Emits {
  (e: 'confirm'): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  selection: null,
  showDimensions: true
});

const emit = defineEmits<Emits>();

const rectangleStyle = computed(() => {
  if (!props.selection) {
    return {
      display: 'none'
    };
  }

  return {
    display: 'block',
    left: props.selection.startX + 'px',
    top: props.selection.startY + 'px',
    width: props.selection.width + 'px',
    height: props.selection.height + 'px'
  };
});

const dimensionText = computed(() => {
  if (!props.selection) return '';
  return `${props.selection.width} × ${props.selection.height}`;
});

function handleConfirm() {
  emit('confirm');
}
</script>

<template>
  <div v-if="visible && selection" class="selection-container">
    <!-- Semi-transparent selection rectangle -->
    <div class="selection-rectangle" :style="rectangleStyle">
      <!-- Border overlay for better visibility -->
      <div class="selection-border"></div>
      
      <!-- Corner handles for resizing -->
      <div class="corner-handle top-left"></div>
      <div class="corner-handle top-right"></div>
      <div class="corner-handle bottom-left"></div>
      <div class="corner-handle bottom-right"></div>
      
      <!-- Dimension label (moved to top-left corner) -->
      <div v-if="showDimensions" class="dimension-label">
        {{ dimensionText }}
      </div>
      
      <!-- Confirm button (positioned below rectangle) -->
      <button class="confirm-button" @click="handleConfirm">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.selection-container {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 10001;
}

.selection-rectangle {
  position: absolute;
  background: rgba(66, 135, 245, 0.2);
  border: 2px solid #4287f5;
  box-sizing: border-box;
  overflow: visible;
}

.selection-border {
  position: absolute;
  inset: -2px;
  border: 1px dashed rgba(66, 135, 245, 0.6);
  pointer-events: none;
}

/* Corner handles for resizing */
.corner-handle {
  position: absolute;
  width: 12px;
  height: 12px;
  background: #4287f5;
  border: 2px solid white;
  border-radius: 2px;
  pointer-events: auto;
  cursor: pointer;
  z-index: 10;
}

.corner-handle.top-left {
  top: -6px;
  left: -6px;
  cursor: nw-resize;
}

.corner-handle.top-right {
  top: -6px;
  right: -6px;
  cursor: ne-resize;
}

.corner-handle.bottom-left {
  bottom: -6px;
  left: -6px;
  cursor: sw-resize;
}

.corner-handle.bottom-right {
  bottom: -6px;
  right: -6px;
  cursor: se-resize;
}

/* Dimension label positioned at top-left corner inside rectangle */
.dimension-label {
  position: absolute;
  top: 0;
  left: 0;
  background: rgba(66, 135, 245, 0.9);
  color: white;
  padding: 4px 8px;
  border-radius: 0 0 6px 0;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  pointer-events: none;
  backdrop-filter: blur(4px);
}

/* Confirm button positioned below rectangle */
.confirm-button {
  position: absolute;
  left: 50%;
  bottom: -48px;
  transform: translateX(-50%);
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: #4287f5;
  border: 3px solid white;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  pointer-events: auto;
  box-shadow: 0 4px 12px rgba(66, 135, 245, 0.4);
  transition: all 0.2s ease;
  padding: 0;
}

.confirm-button:hover {
  background: #3a6ecf;
  transform: translateX(-50%) scale(1.1);
  box-shadow: 0 6px 16px rgba(66, 135, 245, 0.5);
}

.confirm-button:active {
  transform: translateX(-50%) scale(0.95);
}

.confirm-button svg {
  width: 20px;
  height: 20px;
}
</style>

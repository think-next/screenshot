<script setup lang="ts">
import { computed } from 'vue';
import type { Selection } from '../composables/useSelection';

interface Props {
  visible?: boolean
  selection?: Selection | null
  showDimensions?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  selection: null,
  showDimensions: true
});

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
</script>

<template>
  <div v-if="visible && selection" class="selection-container">
    <!-- Semi-transparent selection rectangle -->
    <div class="selection-rectangle" :style="rectangleStyle">
      <!-- Border overlay for better visibility -->
      <div class="selection-border"></div>
      
      <!-- Dimension label -->
      <div v-if="showDimensions" class="dimension-label">
        {{ dimensionText }}
      </div>
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

.dimension-label {
  position: absolute;
  bottom: -30px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.75);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  pointer-events: none;
  backdrop-filter: blur(4px);
}
</style>
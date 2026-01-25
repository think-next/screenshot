<script setup lang="ts">
interface Position {
  x: number
  y: number
}

interface Props {
  visible?: boolean
  position?: Position | null
  color?: string
  thickness?: number
}

withDefaults(defineProps<Props>(), {
  visible: false,
  position: null,
  color: '#ff0000',
  thickness: 1
})
</script>

<template>
  <div v-if="visible && position" class="crosshair-container">
    <div
      class="crosshair-horizontal"
      :style="{ 
        top: position.y + 'px',
        backgroundColor: color,
        height: thickness + 'px'
      }"
    ></div>
    <div
      class="crosshair-vertical"
      :style="{ 
        left: position.x + 'px',
        backgroundColor: color,
        width: thickness + 'px'
      }"
    ></div>
  </div>
</template>

<style scoped>
.crosshair-container {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 10000;
}

.crosshair-horizontal {
  position: fixed;
  left: 0;
  width: 100%;
  transform: translateY(-50%);
}

.crosshair-vertical {
  position: fixed;
  top: 0;
  height: 100%;
  transform: translateX(-50%);
}
</style>

import { ref, onMounted, onBeforeUnmount } from 'vue';

export interface Position {
    x: number
    y: number
}

export function useCrosshair() {
    const position = ref<Position | null>(null);
    let mouseMoveHandler: ((event: MouseEvent) => void) | null = null;

    function startTracking() {
        // Initialize position to screen center
        position.value = {
            x: window.innerWidth / 2,
            y: window.innerHeight / 2
        };

        // Add mouse move listener
        mouseMoveHandler = (event: MouseEvent) => {
            position.value = {
                x: event.clientX,
                y: event.clientY
            };
        };
        window.addEventListener('mousemove', mouseMoveHandler);
    }

    function stopTracking() {
        // Remove mouse move listener
        if (mouseMoveHandler) {
            window.removeEventListener('mousemove', mouseMoveHandler);
            mouseMoveHandler = null;
        }
        // Clear position
        position.value = null;
    }

    // Clean up on unmount
    onBeforeUnmount(() => {
        stopTracking();
    });

    return {
        position,
        startTracking,
        stopTracking
    };
}

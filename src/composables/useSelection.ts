import { ref, onMounted, onBeforeUnmount } from 'vue';

export interface Position {
    x: number
    y: number
}

export interface Selection {
    startX: number
    startY: number
    endX: number
    endY: number
    width: number
    height: number
}

export function useSelection() {
    const isSelecting = ref(false);
    const selection = ref<Selection | null>(null);
    const startPos = ref<Position | null>(null);

    let mouseDownHandler: ((event: MouseEvent) => void) | null = null;
    let mouseMoveHandler: ((event: MouseEvent) => void) | null = null;
    let mouseUpHandler: ((event: MouseEvent) => void) | null = null;

    function startSelection() {
        // Initialize selection state
        isSelecting.value = false;
        selection.value = null;
        startPos.value = null;

        // Add event listeners
        mouseDownHandler = (event: MouseEvent) => {
            // Only start selection with left mouse button
            if (event.button !== 0) return;

            isSelecting.value = true;
            startPos.value = {
                x: event.clientX,
                y: event.clientY
            };

            // Initialize selection with zero size
            selection.value = {
                startX: event.clientX,
                startY: event.clientY,
                endX: event.clientX,
                endY: event.clientY,
                width: 0,
                height: 0
            };
        };

        mouseMoveHandler = (event: MouseEvent) => {
            if (!isSelecting.value || !startPos.value || !selection.value) return;

            const currentX = event.clientX;
            const currentY = event.clientY;

            // Calculate selection dimensions
            const startX = startPos.value.x;
            const startY = startPos.value.y;

            // Ensure width and height are always positive
            const left = Math.min(startX, currentX);
            const top = Math.min(startY, currentY);
            const width = Math.abs(currentX - startX);
            const height = Math.abs(currentY - startY);

            // Update selection
            selection.value = {
                startX: left,
                startY: top,
                endX: currentX,
                endY: currentY,
                width,
                height
            };
        };

        mouseUpHandler = (event: MouseEvent) => {
            if (!isSelecting.value) return;

            // Only trigger on left mouse button release
            if (event.button !== 0) return;

            // Ensure we have a valid selection (minimum size)
            if (selection.value && selection.value.width > 5 && selection.value.height > 5) {
                // Selection is valid, keep it for capture
            } else {
                // Selection too small, clear it
                selection.value = null;
            }

            isSelecting.value = false;
            startPos.value = null;
        };

        window.addEventListener('mousedown', mouseDownHandler);
        window.addEventListener('mousemove', mouseMoveHandler);
        window.addEventListener('mouseup', mouseUpHandler);
    }

    function stopSelection() {
        // Remove event listeners
        if (mouseDownHandler) {
            window.removeEventListener('mousedown', mouseDownHandler);
            mouseDownHandler = null;
        }
        if (mouseMoveHandler) {
            window.removeEventListener('mousemove', mouseMoveHandler);
            mouseMoveHandler = null;
        }
        if (mouseUpHandler) {
            window.removeEventListener('mouseup', mouseUpHandler);
            mouseUpHandler = null;
        }

        // Clear state
        isSelecting.value = false;
        selection.value = null;
        startPos.value = null;
    }

    function clearSelection() {
        selection.value = null;
        isSelecting.value = false;
        startPos.value = null;
    }

    // Clean up on unmount
    onBeforeUnmount(() => {
        stopSelection();
    });

    return {
        isSelecting,
        selection,
        startSelection,
        stopSelection,
        clearSelection
    };
}
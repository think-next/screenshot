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

export type ResizeCorner = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

export function useSelection() {
    const isSelecting = ref(false);
    const isResizing = ref(false);
    const selection = ref<Selection | null>(null);
    const startPos = ref<Position | null>(null);
    const currentResizeCorner = ref<ResizeCorner | null>(null);
    const initialSelection = ref<Selection | null>(null);

    let mouseDownHandler: ((event: MouseEvent) => void) | null = null;
    let mouseMoveHandler: ((event: MouseEvent) => void) | null = null;
    let mouseUpHandler: ((event: MouseEvent) => void) | null = null;

    function startSelection() {
        // Initialize selection state
        isSelecting.value = false;
        isResizing.value = false;
        selection.value = null;
        startPos.value = null;
        currentResizeCorner.value = null;
        initialSelection.value = null;

        // Add event listeners
        mouseDownHandler = (event: MouseEvent) => {
            // Only start selection with left mouse button
            if (event.button !== 0) return;

            // Check if clicking on a corner handle (check target class)
            const target = event.target as HTMLElement;

            // Don't start selection if clicking on the confirm button or inside it
            if (target && (target.classList.contains('confirm-button') ||
                target.closest('.confirm-button'))) {
                // Let the button's click event handler take care of it
                return;
            }

            if (target && target.classList.contains('corner-handle') && selection.value) {
                // Start resizing
                isResizing.value = true;
                currentResizeCorner.value = getResizeCornerFromElement(target);
                initialSelection.value = { ...selection.value };
                startPos.value = {
                    x: event.clientX,
                    y: event.clientY
                };
                return;
            }

            // Start new selection
            isSelecting.value = true;
            isResizing.value = false;
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
            if (isResizing.value && initialSelection.value && currentResizeCorner.value && startPos.value) {
                // Handle resizing
                const deltaX = event.clientX - startPos.value.x;
                const deltaY = event.clientY - startPos.value.y;
                const initial = initialSelection.value;

                let newStartX = initial.startX;
                let newStartY = initial.startY;
                let newEndX = initial.endX;
                let newEndY = initial.endY;

                // Adjust based on which corner is being dragged
                switch (currentResizeCorner.value) {
                    case 'top-left':
                        newStartX = initial.startX + deltaX;
                        newStartY = initial.startY + deltaY;
                        break;
                    case 'top-right':
                        newEndX = initial.endX + deltaX;
                        newStartY = initial.startY + deltaY;
                        break;
                    case 'bottom-left':
                        newStartX = initial.startX + deltaX;
                        newEndY = initial.endY + deltaY;
                        break;
                    case 'bottom-right':
                        newEndX = initial.endX + deltaX;
                        newEndY = initial.endY + deltaY;
                        break;
                }

                // Calculate normalized selection (ensure width and height are positive)
                const left = Math.min(newStartX, newEndX);
                const top = Math.min(newStartY, newEndY);
                const width = Math.abs(newEndX - newStartX);
                const height = Math.abs(newEndY - newStartY);

                // Enforce minimum size
                if (width >= 10 && height >= 10) {
                    selection.value = {
                        startX: left,
                        startY: top,
                        endX: newEndX,
                        endY: newEndY,
                        width,
                        height
                    };
                }
            } else if (isSelecting.value && startPos.value && selection.value) {
                // Handle initial selection creation
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
            }
        };

        mouseUpHandler = (event: MouseEvent) => {
            if (isResizing.value) {
                // Stop resizing
                isResizing.value = false;
                currentResizeCorner.value = null;
                initialSelection.value = null;
                startPos.value = null;
                return;
            }

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

    function getResizeCornerFromElement(element: HTMLElement): ResizeCorner | null {
        if (element.classList.contains('top-left')) return 'top-left';
        if (element.classList.contains('top-right')) return 'top-right';
        if (element.classList.contains('bottom-left')) return 'bottom-left';
        if (element.classList.contains('bottom-right')) return 'bottom-right';
        return null;
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
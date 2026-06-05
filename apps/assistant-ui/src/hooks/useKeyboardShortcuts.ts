import { useEffect, useCallback } from 'react';

interface KeyboardShortcutsOptions {
  onToggleSettings?: () => void;
  onEscape?: () => void;
  onNewChat?: () => void;
  onCopy?: () => void;
  onPaste?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  onSave?: () => void;
  onOpen?: () => void;
  onClose?: () => void;
  onSearch?: () => void;
  onHelp?: () => void;
}

export function useKeyboardShortcuts(options: KeyboardShortcutsOptions) {
  const {
    onToggleSettings,
    onEscape,
    onNewChat,
    onCopy,
    onPaste,
    onUndo,
    onRedo,
    onSave,
    onOpen,
    onClose,
    onSearch,
    onHelp
  } = options;

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    const { key, ctrlKey, metaKey, altKey, shiftKey } = event;
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const cmdOrCtrl = isMac ? metaKey : ctrlKey;

    // Prevent default behavior for our shortcuts
    const preventDefault = () => {
      event.preventDefault();
      event.stopPropagation();
    };

    // Escape key
    if (key === 'Escape' && onEscape) {
      preventDefault();
      onEscape();
      return;
    }

    // Settings toggle (Cmd/Ctrl + ,)
    if (key === ',' && cmdOrCtrl && onToggleSettings) {
      preventDefault();
      onToggleSettings();
      return;
    }

    // New chat (Cmd/Ctrl + N)
    if (key === 'n' && cmdOrCtrl && onNewChat) {
      preventDefault();
      onNewChat();
      return;
    }

    // Copy (Cmd/Ctrl + C)
    if (key === 'c' && cmdOrCtrl && onCopy) {
      preventDefault();
      onCopy();
      return;
    }

    // Paste (Cmd/Ctrl + V)
    if (key === 'v' && cmdOrCtrl && onPaste) {
      preventDefault();
      onPaste();
      return;
    }

    // Undo (Cmd/Ctrl + Z)
    if (key === 'z' && cmdOrCtrl && !shiftKey && onUndo) {
      preventDefault();
      onUndo();
      return;
    }

    // Redo (Cmd/Ctrl + Shift + Z)
    if (key === 'z' && cmdOrCtrl && shiftKey && onRedo) {
      preventDefault();
      onRedo();
      return;
    }

    // Save (Cmd/Ctrl + S)
    if (key === 's' && cmdOrCtrl && onSave) {
      preventDefault();
      onSave();
      return;
    }

    // Open (Cmd/Ctrl + O)
    if (key === 'o' && cmdOrCtrl && onOpen) {
      preventDefault();
      onOpen();
      return;
    }

    // Close (Cmd/Ctrl + W)
    if (key === 'w' && cmdOrCtrl && onClose) {
      preventDefault();
      onClose();
      return;
    }

    // Search (Cmd/Ctrl + F)
    if (key === 'f' && cmdOrCtrl && onSearch) {
      preventDefault();
      onSearch();
      return;
    }

    // Help (F1 or Cmd/Ctrl + ?)
    if ((key === 'F1' || (key === '?' && cmdOrCtrl)) && onHelp) {
      preventDefault();
      onHelp();
      return;
    }

    // Global shortcuts (work even when app is not focused)
    if (isMac) {
      // macOS specific shortcuts
      if (key === 'a' && metaKey && altKey && onToggleSettings) {
        // Cmd+Option+A is handled by the main process
        return;
      }
    } else {
      // Windows/Linux specific shortcuts
      if (key === 'A' && shiftKey && altKey && onToggleSettings) {
        // Shift+Alt+A is handled by the main process
        return;
      }
    }
  }, [
    onToggleSettings,
    onEscape,
    onNewChat,
    onCopy,
    onPaste,
    onUndo,
    onRedo,
    onSave,
    onOpen,
    onClose,
    onSearch,
    onHelp
  ]);

  useEffect(() => {
    // Add event listener
    document.addEventListener('keydown', handleKeyDown);

    // Cleanup
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);

  // Return a function to manually trigger shortcuts (useful for testing)
  const triggerShortcut = useCallback((shortcut: string) => {
    const event = new KeyboardEvent('keydown', {
      key: shortcut,
      ctrlKey: shortcut.includes('Ctrl'),
      metaKey: shortcut.includes('Cmd'),
      altKey: shortcut.includes('Alt'),
      shiftKey: shortcut.includes('Shift')
    });
    
    handleKeyDown(event);
  }, [handleKeyDown]);

  return { triggerShortcut };
}

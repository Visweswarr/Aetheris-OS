import { useState, useCallback } from 'react';
import { ToastProps } from '../types';

export function useNotifications() {
  const [toasts, setToasts] = useState<ToastProps[]>([]);

  const showNotification = useCallback((notification: Omit<ToastProps, 'id'>) => {
    const id = Math.random().toString(36).substr(2, 9);
    const toast: ToastProps = {
      id,
      duration: 5000,
      ...notification
    };

    setToasts(prev => [...prev, toast]);

    // Auto remove after duration
    if (toast.duration && toast.duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, toast.duration);
    }

    return id;
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(toast => toast.id !== id));
  }, []);

  const clearAllToasts = useCallback(() => {
    setToasts([]);
  }, []);

  const showSuccess = useCallback((title: string, message?: string) => {
    return showNotification({
      type: 'success',
      title,
      message
    });
  }, [showNotification]);

  const showError = useCallback((title: string, message?: string) => {
    return showNotification({
      type: 'error',
      title,
      message,
      duration: 8000 // Longer duration for errors
    });
  }, [showNotification]);

  const showWarning = useCallback((title: string, message?: string) => {
    return showNotification({
      type: 'warning',
      title,
      message
    });
  }, [showNotification]);

  const showInfo = useCallback((title: string, message?: string) => {
    return showNotification({
      type: 'info',
      title,
      message
    });
  }, [showNotification]);

  return {
    toasts,
    showNotification,
    removeToast,
    clearAllToasts,
    showSuccess,
    showError,
    showWarning,
    showInfo
  };
}

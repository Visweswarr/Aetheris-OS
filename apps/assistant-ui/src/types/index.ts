export interface ChatMessage {
  id: string;
  content: string;
  role: 'user' | 'assistant';
  timestamp: Date;
  metadata?: {
    citations?: string[];
    tools_used?: string[];
    confidence?: number;
    processing_time_ms?: number;
  };
}

export interface ChatResponse {
  id: string;
  content: string;
  role: 'assistant';
  timestamp: Date;
  citations?: string[];
  tools_used?: string[];
  metadata?: {
    confidence?: number;
    processing_time_ms?: number;
    model_version?: string;
    usage_tokens?: number;
  };
}

export interface ToolCall {
  name: string;
  parameters: any;
  result?: any;
  error?: string;
  execution_time_ms?: number;
}

export interface MemoryEntry {
  key: string;
  value: any;
  timestamp: Date;
  ttl?: number;
  tags?: string[];
}

export interface AppState {
  isVisible: boolean;
  version: string;
  platform: string;
  isConnected: boolean;
  messageHistory: ChatMessage[];
}

export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface StreamingChunk {
  message_id: string;
  chunk: string;
  is_complete: boolean;
  timestamp: Date;
}

export interface Citation {
  id: string;
  title: string;
  url: string;
  snippet: string;
  relevance_score: number;
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: any;
  required_capabilities: string[];
  category: string;
  tags: string[];
}

export interface AIStats {
  total_messages: number;
  total_tokens: number;
  avg_response_time_ms: number;
  tools_called: number;
  memory_entries: number;
  uptime_ms: number;
}

export interface ErrorInfo {
  code: string;
  message: string;
  details?: any;
  timestamp: Date;
}

export interface Settings {
  theme: 'light' | 'dark' | 'auto';
  fontSize: 'small' | 'medium' | 'large';
  maxHistory: number;
  autoHide: boolean;
  hotkey: string;
  windowPosition: 'center' | 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left';
  enableStreaming: boolean;
  enableCitations: boolean;
  enableTools: boolean;
  enableMemory: boolean;
}

export interface NotificationOptions {
  title: string;
  body: string;
  icon?: string;
  silent?: boolean;
  timeout?: number;
}

export interface KeyboardShortcut {
  key: string;
  modifiers: string[];
  action: string;
  description: string;
}

export interface Theme {
  name: string;
  colors: {
    primary: string;
    secondary: string;
    background: string;
    surface: string;
    text: string;
    textSecondary: string;
    border: string;
    accent: string;
    success: string;
    warning: string;
    error: string;
  };
  fonts: {
    primary: string;
    mono: string;
  };
  spacing: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
  };
  borderRadius: {
    sm: string;
    md: string;
    lg: string;
  };
  shadows: {
    sm: string;
    md: string;
    lg: string;
  };
}

export interface ComponentProps {
  className?: string;
  children?: React.ReactNode;
}

export interface ButtonProps extends ComponentProps {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  onClick?: () => void;
  type?: 'button' | 'submit' | 'reset';
}

export interface InputProps extends ComponentProps {
  type?: 'text' | 'email' | 'password' | 'number' | 'search';
  placeholder?: string;
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  required?: boolean;
  onChange?: (value: string) => void;
  onFocus?: () => void;
  onBlur?: () => void;
  onKeyDown?: (event: React.KeyboardEvent) => void;
}

export interface TextareaProps extends ComponentProps {
  placeholder?: string;
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  required?: boolean;
  rows?: number;
  maxLength?: number;
  onChange?: (value: string) => void;
  onFocus?: () => void;
  onBlur?: () => void;
  onKeyDown?: (event: React.KeyboardEvent) => void;
}

export interface ModalProps extends ComponentProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closable?: boolean;
}

export interface DropdownProps extends ComponentProps {
  options: Array<{
    value: string;
    label: string;
    disabled?: boolean;
  }>;
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  onChange?: (value: string) => void;
}

export interface ToastProps {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export interface LoadingState {
  isLoading: boolean;
  message?: string;
  progress?: number;
}

export interface SearchResult {
  id: string;
  title: string;
  content: string;
  url?: string;
  relevance_score: number;
  timestamp: Date;
}

export interface FileUpload {
  id: string;
  name: string;
  size: number;
  type: string;
  data: ArrayBuffer;
  progress: number;
  status: 'uploading' | 'completed' | 'error';
  error?: string;
}

import React, { useState, useEffect, useCallback } from 'react';
import './posix_ui.css';

// Types for POSIX data structures
interface ProcessInfo {
  pid: number;
  parentPid?: number;
  state: 'Running' | 'Sleeping' | 'Zombie' | 'Stopped' | 'Terminated';
  executable: string;
  createdAt: number;
  cpuTime: number;
  memoryUsage: number;
  ngfsSnapshot?: string;
}

interface SignalInfo {
  signal: string;
  targetPid: number;
  senderPid?: number;
  timestamp: number;
  data?: string;
}

interface IPCInfo {
  type: 'pipe' | 'message_queue' | 'shared_memory';
  id: string;
  name: string;
  size?: number;
  messages?: number;
  attachedProcesses?: number;
}

interface ThreadInfo {
  tid: number;
  pid: number;
  state: 'Ready' | 'Running' | 'Blocked' | 'Sleeping' | 'Terminated';
  function: string;
  priority: number;
  cpuQuota: number;
}

interface PerformanceMetrics {
  processForkP50: number;
  processForkP95: number;
  signalDeliveryP50: number;
  signalDeliveryP95: number;
  ipcPipeRoundtripP50: number;
  ipcPipeRoundtripP95: number;
}

interface POSIXStatus {
  brokerReady: boolean;
  vfsReady: boolean;
  shimsReady: boolean;
  shellReady: boolean;
  mountCount: number;
  fileCount: number;
  shimCount: number;
  brokerCalls: number;
}

// Main POSIX Management UI Component
export const POSIXUI: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'processes' | 'signals' | 'ipc' | 'threads' | 'performance'>('processes');
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [signals, setSignals] = useState<SignalInfo[]>([]);
  const [ipcObjects, setIpcObjects] = useState<IPCInfo[]>([]);
  const [threads, setThreads] = useState<ThreadInfo[]>([]);
  const [performance, setPerformance] = useState<PerformanceMetrics>({
    processForkP50: 0,
    processForkP95: 0,
    signalDeliveryP50: 0,
    signalDeliveryP95: 0,
    ipcPipeRoundtripP50: 0,
    ipcPipeRoundtripP95: 0
  });
  const [status, setStatus] = useState<POSIXStatus>({
    brokerReady: false,
    vfsReady: false,
    shimsReady: false,
    shellReady: false,
    mountCount: 0,
    fileCount: 0,
    shimCount: 0,
    brokerCalls: 0
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Mock data for demonstration
  const mockProcesses: ProcessInfo[] = [
    {
      pid: 1,
      state: 'Running',
      executable: '/sbin/init',
      createdAt: Date.now() - 3600000,
      cpuTime: 0,
      memoryUsage: 1024
    },
    {
      pid: 2,
      parentPid: 1,
      state: 'Running',
      executable: '/bin/bash',
      createdAt: Date.now() - 1800000,
      cpuTime: 150,
      memoryUsage: 2048
    },
    {
      pid: 3,
      parentPid: 2,
      state: 'Sleeping',
      executable: '/bin/sleep',
      createdAt: Date.now() - 300000,
      cpuTime: 5,
      memoryUsage: 512
    }
  ];

  const mockSignals: SignalInfo[] = [
    {
      signal: 'SIGUSR1',
      targetPid: 2,
      senderPid: 1,
      timestamp: Date.now() - 60000,
      data: 'User signal 1'
    },
    {
      signal: 'SIGINT',
      targetPid: 3,
      senderPid: 2,
      timestamp: Date.now() - 30000,
      data: 'Ctrl+C'
    }
  ];

  const mockIPC: IPCInfo[] = [
    {
      type: 'pipe',
      id: 'pipe_1',
      name: 'stdout_pipe',
      size: 65536
    },
    {
      type: 'message_queue',
      id: 'mq_1',
      name: 'worker_queue',
      messages: 5
    },
    {
      type: 'shared_memory',
      id: 'shm_1',
      name: 'data_buffer',
      size: 4096,
      attachedProcesses: 2
    }
  ];

  const mockThreads: ThreadInfo[] = [
    {
      tid: 2001,
      pid: 2,
      state: 'Running',
      function: 'main',
      priority: 4,
      cpuQuota: 1000
    },
    {
      tid: 2002,
      pid: 2,
      state: 'Ready',
      function: 'worker',
      priority: 3,
      cpuQuota: 500
    }
  ];

  // Load data on component mount
  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      // In a real implementation, these would be API calls to the POSIX service
      // For now, we'll use mock data
      await new Promise(resolve => setTimeout(resolve, 500)); // Simulate API delay
      
      setProcesses(mockProcesses);
      setSignals(mockSignals);
      setIpcObjects(mockIPC);
      setThreads(mockThreads);
      setPerformance({
        processForkP50: 0.3,
        processForkP95: 1.2,
        signalDeliveryP50: 0.15,
        signalDeliveryP95: 0.6,
        ipcPipeRoundtripP50: 0.25,
        ipcPipeRoundtripP95: 0.8
      });
      setStatus({
        brokerReady: true,
        vfsReady: true,
        shimsReady: true,
        shellReady: true,
        mountCount: 4,
        fileCount: 12,
        shimCount: 5,
        brokerCalls: 42
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  }, []);

  const handleProcessAction = async (action: string, pid: number, ...args: any[]) => {
    setLoading(true);
    try {
      // In a real implementation, this would call the POSIX service API
      console.log(`Process action: ${action} on PID ${pid}`, args);
      await new Promise(resolve => setTimeout(resolve, 200));
      
      // Update local state based on action
      if (action === 'kill') {
        setProcesses(prev => prev.map(p => 
          p.pid === pid ? { ...p, state: 'Terminated' as const } : p
        ));
      } else if (action === 'fork') {
        const newProcess: ProcessInfo = {
          pid: Math.max(...processes.map(p => p.pid)) + 1,
          parentPid: pid,
          state: 'Running',
          executable: args[0] || '/bin/forked',
          createdAt: Date.now(),
          cpuTime: 0,
          memoryUsage: 1024
        };
        setProcesses(prev => [...prev, newProcess]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Action failed');
    } finally {
      setLoading(false);
    }
  };

  const handleSignalSend = async (pid: number, signal: string) => {
    setLoading(true);
    try {
      console.log(`Sending signal ${signal} to PID ${pid}`);
      await new Promise(resolve => setTimeout(resolve, 100));
      
      const newSignal: SignalInfo = {
        signal,
        targetPid: pid,
        timestamp: Date.now()
      };
      setSignals(prev => [newSignal, ...prev]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Signal send failed');
    } finally {
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDuration = (ms: number): string => {
    if (ms < 1) return `${(ms * 1000).toFixed(0)}µs`;
    return `${ms.toFixed(2)}ms`;
  };

  const getStateColor = (state: string): string => {
    switch (state) {
      case 'Running': return '#4CAF50';
      case 'Sleeping': return '#FF9800';
      case 'Stopped': return '#F44336';
      case 'Terminated': return '#9E9E9E';
      case 'Zombie': return '#795548';
      default: return '#2196F3';
    }
  };

  return (
    <div className="posix-ui">
      <div className="posix-header">
        <h1>🚀 POSIX Advanced Features Management</h1>
        <div className="status-indicators">
          <div className={`status-indicator ${status.brokerReady ? 'ready' : 'not-ready'}`}>
            Broker {status.brokerReady ? '✅' : '❌'}
          </div>
          <div className={`status-indicator ${status.vfsReady ? 'ready' : 'not-ready'}`}>
            VFS {status.vfsReady ? '✅' : '❌'}
          </div>
          <div className={`status-indicator ${status.shimsReady ? 'ready' : 'not-ready'}`}>
            Shims {status.shimsReady ? '✅' : '❌'}
          </div>
          <div className={`status-indicator ${status.shellReady ? 'ready' : 'not-ready'}`}>
            Shell {status.shellReady ? '✅' : '❌'}
          </div>
        </div>
      </div>

      {error && (
        <div className="error-banner">
          ❌ Error: {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="posix-tabs">
        {(['processes', 'signals', 'ipc', 'threads', 'performance'] as const).map(tab => (
          <button
            key={tab}
            className={`tab ${activeTab === tab ? 'active' : ''}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div className="posix-content">
        {loading && <div className="loading-spinner">⏳ Loading...</div>}

        {activeTab === 'processes' && (
          <ProcessManagement
            processes={processes}
            onProcessAction={handleProcessAction}
            loading={loading}
          />
        )}

        {activeTab === 'signals' && (
          <SignalManagement
            signals={signals}
            processes={processes}
            onSendSignal={handleSignalSend}
            loading={loading}
          />
        )}

        {activeTab === 'ipc' && (
          <IPCManagement
            ipcObjects={ipcObjects}
            loading={loading}
          />
        )}

        {activeTab === 'threads' && (
          <ThreadManagement
            threads={threads}
            processes={processes}
            loading={loading}
          />
        )}

        {activeTab === 'performance' && (
          <PerformanceDashboard
            metrics={performance}
            status={status}
            loading={loading}
          />
        )}
      </div>
    </div>
  );
};

// Process Management Component
const ProcessManagement: React.FC<{
  processes: ProcessInfo[];
  onProcessAction: (action: string, pid: number, ...args: any[]) => void;
  loading: boolean;
}> = ({ processes, onProcessAction, loading }) => {
  const [selectedPid, setSelectedPid] = useState<number | null>(null);
  const [forkExecutable, setForkExecutable] = useState('');

  return (
    <div className="process-management">
      <div className="section-header">
        <h2>🔄 Process Management</h2>
        <div className="action-buttons">
          <button
            onClick={() => onProcessAction('refresh', 0)}
            disabled={loading}
          >
            🔄 Refresh
          </button>
        </div>
      </div>

      <div className="process-actions">
        <div className="action-group">
          <label>Fork Process:</label>
          <input
            type="text"
            value={forkExecutable}
            onChange={(e) => setForkExecutable(e.target.value)}
            placeholder="/bin/executable"
          />
          <button
            onClick={() => {
              if (forkExecutable) {
                onProcessAction('fork', 1, forkExecutable);
                setForkExecutable('');
              }
            }}
            disabled={loading || !forkExecutable}
          >
            🍴 Fork
          </button>
        </div>
      </div>

      <div className="process-table">
        <table>
          <thead>
            <tr>
              <th>PID</th>
              <th>PPID</th>
              <th>State</th>
              <th>Executable</th>
              <th>CPU Time</th>
              <th>Memory</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {processes.map(process => (
              <tr key={process.pid} className={selectedPid === process.pid ? 'selected' : ''}>
                <td>{process.pid}</td>
                <td>{process.parentPid || '-'}</td>
                <td>
                  <span 
                    className="state-badge"
                    style={{ backgroundColor: getStateColor(process.state) }}
                  >
                    {process.state}
                  </span>
                </td>
                <td>{process.executable}</td>
                <td>{process.cpuTime}ms</td>
                <td>{formatBytes(process.memoryUsage)}</td>
                <td>
                  <div className="action-buttons">
                    <button
                      onClick={() => onProcessAction('kill', process.pid, 'SIGTERM')}
                      disabled={loading || process.state === 'Terminated'}
                      className="kill-btn"
                    >
                      💀 Kill
                    </button>
                    <button
                      onClick={() => onProcessAction('signal', process.pid, 'SIGUSR1')}
                      disabled={loading || process.state === 'Terminated'}
                    >
                      📡 Signal
                    </button>
                    <button
                      onClick={() => setSelectedPid(process.pid)}
                      className="info-btn"
                    >
                      ℹ️ Info
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

// Signal Management Component
const SignalManagement: React.FC<{
  signals: SignalInfo[];
  processes: ProcessInfo[];
  onSendSignal: (pid: number, signal: string) => void;
  loading: boolean;
}> = ({ signals, processes, onSendSignal, loading }) => {
  const [selectedPid, setSelectedPid] = useState<number>(1);
  const [selectedSignal, setSelectedSignal] = useState<string>('SIGUSR1');

  const commonSignals = ['SIGINT', 'SIGTERM', 'SIGKILL', 'SIGUSR1', 'SIGUSR2', 'SIGSTOP', 'SIGCONT'];

  return (
    <div className="signal-management">
      <div className="section-header">
        <h2>📡 Signal Management</h2>
      </div>

      <div className="signal-sender">
        <div className="form-group">
          <label>Target Process:</label>
          <select
            value={selectedPid}
            onChange={(e) => setSelectedPid(Number(e.target.value))}
          >
            {processes.map(process => (
              <option key={process.pid} value={process.pid}>
                PID {process.pid} - {process.executable}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label>Signal:</label>
          <select
            value={selectedSignal}
            onChange={(e) => setSelectedSignal(e.target.value)}
          >
            {commonSignals.map(signal => (
              <option key={signal} value={signal}>{signal}</option>
            ))}
          </select>
        </div>

        <button
          onClick={() => onSendSignal(selectedPid, selectedSignal)}
          disabled={loading}
          className="send-signal-btn"
        >
          📤 Send Signal
        </button>
      </div>

      <div className="signal-history">
        <h3>Signal History</h3>
        <div className="signal-list">
          {signals.map((signal, index) => (
            <div key={index} className="signal-item">
              <div className="signal-info">
                <span className="signal-name">{signal.signal}</span>
                <span className="signal-target">→ PID {signal.targetPid}</span>
                {signal.senderPid && (
                  <span className="signal-sender">from PID {signal.senderPid}</span>
                )}
              </div>
              <div className="signal-meta">
                <span className="signal-time">
                  {new Date(signal.timestamp).toLocaleTimeString()}
                </span>
                {signal.data && (
                  <span className="signal-data">{signal.data}</span>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

// IPC Management Component
const IPCManagement: React.FC<{
  ipcObjects: IPCInfo[];
  loading: boolean;
}> = ({ ipcObjects, loading }) => {
  return (
    <div className="ipc-management">
      <div className="section-header">
        <h2>🔗 IPC Management</h2>
      </div>

      <div className="ipc-stats">
        <div className="stat-card">
          <h3>Pipes</h3>
          <div className="stat-value">
            {ipcObjects.filter(obj => obj.type === 'pipe').length}
          </div>
        </div>
        <div className="stat-card">
          <h3>Message Queues</h3>
          <div className="stat-value">
            {ipcObjects.filter(obj => obj.type === 'message_queue').length}
          </div>
        </div>
        <div className="stat-card">
          <h3>Shared Memory</h3>
          <div className="stat-value">
            {ipcObjects.filter(obj => obj.type === 'shared_memory').length}
          </div>
        </div>
      </div>

      <div className="ipc-objects">
        {ipcObjects.map(obj => (
          <div key={obj.id} className="ipc-object">
            <div className="ipc-header">
              <span className="ipc-type">{obj.type}</span>
              <span className="ipc-name">{obj.name}</span>
              <span className="ipc-id">ID: {obj.id}</span>
            </div>
            <div className="ipc-details">
              {obj.size && <span>Size: {formatBytes(obj.size)}</span>}
              {obj.messages !== undefined && <span>Messages: {obj.messages}</span>}
              {obj.attachedProcesses !== undefined && (
                <span>Attached: {obj.attachedProcesses}</span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

// Thread Management Component
const ThreadManagement: React.FC<{
  threads: ThreadInfo[];
  processes: ProcessInfo[];
  loading: boolean;
}> = ({ threads, processes, loading }) => {
  return (
    <div className="thread-management">
      <div className="section-header">
        <h2>🧵 Thread Management</h2>
      </div>

      <div className="thread-table">
        <table>
          <thead>
            <tr>
              <th>TID</th>
              <th>PID</th>
              <th>State</th>
              <th>Function</th>
              <th>Priority</th>
              <th>CPU Quota</th>
            </tr>
          </thead>
          <tbody>
            {threads.map(thread => (
              <tr key={thread.tid}>
                <td>{thread.tid}</td>
                <td>{thread.pid}</td>
                <td>
                  <span 
                    className="state-badge"
                    style={{ backgroundColor: getStateColor(thread.state) }}
                  >
                    {thread.state}
                  </span>
                </td>
                <td>{thread.function}</td>
                <td>{thread.priority}</td>
                <td>{thread.cpuQuota}ms</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

// Performance Dashboard Component
const PerformanceDashboard: React.FC<{
  metrics: PerformanceMetrics;
  status: POSIXStatus;
  loading: boolean;
}> = ({ metrics, status, loading }) => {
  const performanceBudgets = {
    processForkP50: 0.5,
    processForkP95: 1.5,
    signalDeliveryP50: 0.2,
    signalDeliveryP95: 0.8,
    ipcPipeRoundtripP50: 0.4,
    ipcPipeRoundtripP95: 1.0
  };

  const isWithinBudget = (value: number, budget: number): boolean => {
    return value <= budget;
  };

  return (
    <div className="performance-dashboard">
      <div className="section-header">
        <h2>📊 Performance Dashboard</h2>
      </div>

      <div className="performance-metrics">
        <div className="metric-group">
          <h3>Process Management</h3>
          <div className="metric-item">
            <span className="metric-label">Process Fork (p50):</span>
            <span className={`metric-value ${isWithinBudget(metrics.processForkP50, performanceBudgets.processForkP50) ? 'good' : 'bad'}`}>
              {formatDuration(metrics.processForkP50)}
            </span>
            <span className="metric-budget">(≤ {formatDuration(performanceBudgets.processForkP50)})</span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Process Fork (p95):</span>
            <span className={`metric-value ${isWithinBudget(metrics.processForkP95, performanceBudgets.processForkP95) ? 'good' : 'bad'}`}>
              {formatDuration(metrics.processForkP95)}
            </span>
            <span className="metric-budget">(≤ {formatDuration(performanceBudgets.processForkP95)})</span>
          </div>
        </div>

        <div className="metric-group">
          <h3>Signal Handling</h3>
          <div className="metric-item">
            <span className="metric-label">Signal Delivery (p50):</span>
            <span className={`metric-value ${isWithinBudget(metrics.signalDeliveryP50, performanceBudgets.signalDeliveryP50) ? 'good' : 'bad'}`}>
              {formatDuration(metrics.signalDeliveryP50)}
            </span>
            <span className="metric-budget">(≤ {formatDuration(performanceBudgets.signalDeliveryP50)})</span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Signal Delivery (p95):</span>
            <span className={`metric-value ${isWithinBudget(metrics.signalDeliveryP95, performanceBudgets.signalDeliveryP95) ? 'good' : 'bad'}`}>
              {formatDuration(metrics.signalDeliveryP95)}
            </span>
            <span className="metric-budget">(≤ {formatDuration(performanceBudgets.signalDeliveryP95)})</span>
          </div>
        </div>

        <div className="metric-group">
          <h3>IPC Performance</h3>
          <div className="metric-item">
            <span className="metric-label">Pipe Roundtrip (p50):</span>
            <span className={`metric-value ${isWithinBudget(metrics.ipcPipeRoundtripP50, performanceBudgets.ipcPipeRoundtripP50) ? 'good' : 'bad'}`}>
              {formatDuration(metrics.ipcPipeRoundtripP50)}
            </span>
            <span className="metric-budget">(≤ {formatDuration(performanceBudgets.ipcPipeRoundtripP50)})</span>
          </div>
          <div className="metric-item">
            <span className="metric-label">Pipe Roundtrip (p95):</span>
            <span className={`metric-value ${isWithinBudget(metrics.ipcPipeRoundtripP95, performanceBudgets.ipcPipeRoundtripP95) ? 'good' : 'bad'}`}>
              {formatDuration(metrics.ipcPipeRoundtripP95)}
            </span>
            <span className="metric-budget">(≤ {formatDuration(performanceBudgets.ipcPipeRoundtripP95)})</span>
          </div>
        </div>
      </div>

      <div className="system-stats">
        <h3>System Statistics</h3>
        <div className="stats-grid">
          <div className="stat-item">
            <span className="stat-label">Mount Points:</span>
            <span className="stat-value">{status.mountCount}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Files:</span>
            <span className="stat-value">{status.fileCount}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Shims:</span>
            <span className="stat-value">{status.shimCount}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Broker Calls:</span>
            <span className="stat-value">{status.brokerCalls}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper functions (these would be defined in the main component scope)
const getStateColor = (state: string): string => {
  switch (state) {
    case 'Running': return '#4CAF50';
    case 'Sleeping': return '#FF9800';
    case 'Stopped': return '#F44336';
    case 'Terminated': return '#9E9E9E';
    case 'Zombie': return '#795548';
    default: return '#2196F3';
  }
};

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const formatDuration = (ms: number): string => {
  if (ms < 1) return `${(ms * 1000).toFixed(0)}µs`;
  return `${ms.toFixed(2)}ms`;
};

export default POSIXUI;


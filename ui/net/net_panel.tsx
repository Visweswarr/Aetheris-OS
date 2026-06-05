/**
 * Aetheris OS Network Panel
 * 
 * React component for managing network namespaces, policies, sockets, and flows
 * in the Aetheris OS networking subsystem.
 */

import React, { useState, useEffect, useCallback } from 'react';
import './net_panel.css';

// Types
interface NetworkNamespace {
  id: string;
  name: string;
  class: string;
  created_at: string;
  processes: number;
  sockets: number;
  bytes_sent: number;
  bytes_received: number;
}

interface SocketInfo {
  id: string;
  fd: number;
  type: string;
  state: string;
  local_addr?: string;
  remote_addr?: string;
  process_cap: string;
  namespace: string;
  created_at: string;
  bytes_sent: number;
  bytes_received: number;
}

interface NetworkFlow {
  id: string;
  source_addr: string;
  source_port: number;
  dest_addr: string;
  dest_port: number;
  protocol: string;
  state: string;
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
  start_time: string;
  last_activity: string;
}

interface PolicyInfo {
  id: string;
  name: string;
  description: string;
  priority: number;
  enabled: boolean;
  rules: string[];
  created_at: string;
  updated_at: string;
}

interface NetworkStats {
  total_namespaces: number;
  active_namespaces: number;
  total_sockets: number;
  active_sockets: number;
  total_flows: number;
  active_flows: number;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_packets_sent: number;
  total_packets_received: number;
  policy_decisions: number;
  policy_denials: number;
}

// API client
class NetworkAPI {
  private baseUrl: string;

  constructor(baseUrl: string = '/api/net') {
    this.baseUrl = baseUrl;
  }

  async getNamespaces(): Promise<NetworkNamespace[]> {
    const response = await fetch(`${this.baseUrl}/namespaces`);
    if (!response.ok) {
      throw new Error(`Failed to fetch namespaces: ${response.statusText}`);
    }
    return response.json();
  }

  async createNamespace(name: string, classType: string): Promise<NetworkNamespace> {
    const response = await fetch(`${this.baseUrl}/namespaces`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name, class: classType }),
    });
    if (!response.ok) {
      throw new Error(`Failed to create namespace: ${response.statusText}`);
    }
    return response.json();
  }

  async deleteNamespace(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/namespaces/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      throw new Error(`Failed to delete namespace: ${response.statusText}`);
    }
  }

  async getSockets(namespace?: string): Promise<SocketInfo[]> {
    const url = namespace ? `${this.baseUrl}/sockets?namespace=${namespace}` : `${this.baseUrl}/sockets`;
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch sockets: ${response.statusText}`);
    }
    return response.json();
  }

  async getFlows(namespace?: string): Promise<NetworkFlow[]> {
    const url = namespace ? `${this.baseUrl}/flows?namespace=${namespace}` : `${this.baseUrl}/flows`;
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch flows: ${response.statusText}`);
    }
    return response.json();
  }

  async getPolicies(): Promise<PolicyInfo[]> {
    const response = await fetch(`${this.baseUrl}/policies`);
    if (!response.ok) {
      throw new Error(`Failed to fetch policies: ${response.statusText}`);
    }
    return response.json();
  }

  async getStats(): Promise<NetworkStats> {
    const response = await fetch(`${this.baseUrl}/stats`);
    if (!response.ok) {
      throw new Error(`Failed to fetch stats: ${response.statusText}`);
    }
    return response.json();
  }

  async revokeCapability(processCap: string, capability: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/capabilities/revoke`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ process_cap: processCap, capability }),
    });
    if (!response.ok) {
      throw new Error(`Failed to revoke capability: ${response.statusText}`);
    }
  }
}

// Utility functions
const formatBytes = (bytes: number): string => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
};

const formatTime = (timestamp: string): string => {
  return new Date(timestamp).toLocaleString();
};

const getStateColor = (state: string): string => {
  switch (state.toLowerCase()) {
    case 'connected':
    case 'established':
    case 'listening':
      return '#4caf50';
    case 'connecting':
    case 'syn_sent':
      return '#ff9800';
    case 'disconnected':
    case 'closed':
      return '#f44336';
    case 'error':
      return '#f44336';
    default:
      return '#9e9e9e';
  }
};

// Main component
const NetworkPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'overview' | 'namespaces' | 'sockets' | 'flows' | 'policies'>('overview');
  const [namespaces, setNamespaces] = useState<NetworkNamespace[]>([]);
  const [sockets, setSockets] = useState<SocketInfo[]>([]);
  const [flows, setFlows] = useState<NetworkFlow[]>([]);
  const [policies, setPolicies] = useState<PolicyInfo[]>([]);
  const [stats, setStats] = useState<NetworkStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedNamespace, setSelectedNamespace] = useState<string>('');

  const api = new NetworkAPI();

  // Load data
  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [namespacesData, socketsData, flowsData, policiesData, statsData] = await Promise.all([
        api.getNamespaces(),
        api.getSockets(selectedNamespace || undefined),
        api.getFlows(selectedNamespace || undefined),
        api.getPolicies(),
        api.getStats(),
      ]);

      setNamespaces(namespacesData);
      setSockets(socketsData);
      setFlows(flowsData);
      setPolicies(policiesData);
      setStats(statsData);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [selectedNamespace]);

  // Load data on mount and when selected namespace changes
  useEffect(() => {
    loadData();
  }, [loadData]);

  // Auto-refresh every 5 seconds
  useEffect(() => {
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, [loadData]);

  // Handle namespace creation
  const handleCreateNamespace = async (name: string, classType: string) => {
    try {
      await api.createNamespace(name, classType);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create namespace');
    }
  };

  // Handle namespace deletion
  const handleDeleteNamespace = async (id: string) => {
    if (window.confirm('Are you sure you want to delete this namespace?')) {
      try {
        await api.deleteNamespace(id);
        await loadData();
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to delete namespace');
      }
    }
  };

  // Handle capability revocation
  const handleRevokeCapability = async (processCap: string, capability: string) => {
    if (window.confirm(`Are you sure you want to revoke capability '${capability}' from process '${processCap}'?`)) {
      try {
        await api.revokeCapability(processCap, capability);
        await loadData();
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to revoke capability');
      }
    }
  };

  // Overview tab
  const OverviewTab: React.FC = () => (
    <div className="overview-tab">
      {stats && (
        <div className="stats-grid">
          <div className="stat-card">
            <h3>Namespaces</h3>
            <div className="stat-value">{stats.active_namespaces}/{stats.total_namespaces}</div>
            <div className="stat-label">Active/Total</div>
          </div>
          <div className="stat-card">
            <h3>Sockets</h3>
            <div className="stat-value">{stats.active_sockets}/{stats.total_sockets}</div>
            <div className="stat-label">Active/Total</div>
          </div>
          <div className="stat-card">
            <h3>Flows</h3>
            <div className="stat-value">{stats.active_flows}/{stats.total_flows}</div>
            <div className="stat-label">Active/Total</div>
          </div>
          <div className="stat-card">
            <h3>Data Sent</h3>
            <div className="stat-value">{formatBytes(stats.total_bytes_sent)}</div>
            <div className="stat-label">Total</div>
          </div>
          <div className="stat-card">
            <h3>Data Received</h3>
            <div className="stat-value">{formatBytes(stats.total_bytes_received)}</div>
            <div className="stat-label">Total</div>
          </div>
          <div className="stat-card">
            <h3>Policy Decisions</h3>
            <div className="stat-value">{stats.policy_decisions}</div>
            <div className="stat-label">Total</div>
          </div>
        </div>
      )}
    </div>
  );

  // Namespaces tab
  const NamespacesTab: React.FC = () => (
    <div className="namespaces-tab">
      <div className="tab-header">
        <h2>Network Namespaces</h2>
        <button 
          className="btn btn-primary"
          onClick={() => {
            const name = prompt('Enter namespace name:');
            const classType = prompt('Enter namespace class (none, local, mesh, wan):');
            if (name && classType) {
              handleCreateNamespace(name, classType);
            }
          }}
        >
          Create Namespace
        </button>
      </div>
      <div className="table-container">
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Class</th>
              <th>Processes</th>
              <th>Sockets</th>
              <th>Data Sent</th>
              <th>Data Received</th>
              <th>Created</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {namespaces.map((ns) => (
              <tr key={ns.id}>
                <td>{ns.id}</td>
                <td>{ns.name}</td>
                <td>
                  <span className={`namespace-class namespace-class-${ns.class}`}>
                    {ns.class}
                  </span>
                </td>
                <td>{ns.processes}</td>
                <td>{ns.sockets}</td>
                <td>{formatBytes(ns.bytes_sent)}</td>
                <td>{formatBytes(ns.bytes_received)}</td>
                <td>{formatTime(ns.created_at)}</td>
                <td>
                  <button 
                    className="btn btn-danger btn-sm"
                    onClick={() => handleDeleteNamespace(ns.id)}
                    disabled={ns.id === 'default'}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  // Sockets tab
  const SocketsTab: React.FC = () => (
    <div className="sockets-tab">
      <div className="tab-header">
        <h2>Network Sockets</h2>
        <select 
          value={selectedNamespace}
          onChange={(e) => setSelectedNamespace(e.target.value)}
          className="namespace-filter"
        >
          <option value="">All Namespaces</option>
          {namespaces.map((ns) => (
            <option key={ns.id} value={ns.id}>{ns.name}</option>
          ))}
        </select>
      </div>
      <div className="table-container">
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>FD</th>
              <th>Type</th>
              <th>State</th>
              <th>Local Address</th>
              <th>Remote Address</th>
              <th>Process Cap</th>
              <th>Namespace</th>
              <th>Data Sent</th>
              <th>Data Received</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {sockets.map((socket) => (
              <tr key={socket.id}>
                <td>{socket.id}</td>
                <td>{socket.fd}</td>
                <td>{socket.type}</td>
                <td>
                  <span 
                    className="socket-state"
                    style={{ color: getStateColor(socket.state) }}
                  >
                    {socket.state}
                  </span>
                </td>
                <td>{socket.local_addr || '-'}</td>
                <td>{socket.remote_addr || '-'}</td>
                <td>{socket.process_cap}</td>
                <td>{socket.namespace}</td>
                <td>{formatBytes(socket.bytes_sent)}</td>
                <td>{formatBytes(socket.bytes_received)}</td>
                <td>{formatTime(socket.created_at)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  // Flows tab
  const FlowsTab: React.FC = () => (
    <div className="flows-tab">
      <div className="tab-header">
        <h2>Network Flows</h2>
        <select 
          value={selectedNamespace}
          onChange={(e) => setSelectedNamespace(e.target.value)}
          className="namespace-filter"
        >
          <option value="">All Namespaces</option>
          {namespaces.map((ns) => (
            <option key={ns.id} value={ns.id}>{ns.name}</option>
          ))}
        </select>
      </div>
      <div className="table-container">
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Source</th>
              <th>Destination</th>
              <th>Protocol</th>
              <th>State</th>
              <th>Data Sent</th>
              <th>Data Received</th>
              <th>Packets Sent</th>
              <th>Packets Received</th>
              <th>Start Time</th>
              <th>Last Activity</th>
            </tr>
          </thead>
          <tbody>
            {flows.map((flow) => (
              <tr key={flow.id}>
                <td>{flow.id}</td>
                <td>{flow.source_addr}:{flow.source_port}</td>
                <td>{flow.dest_addr}:{flow.dest_port}</td>
                <td>{flow.protocol}</td>
                <td>
                  <span 
                    className="flow-state"
                    style={{ color: getStateColor(flow.state) }}
                  >
                    {flow.state}
                  </span>
                </td>
                <td>{formatBytes(flow.bytes_sent)}</td>
                <td>{formatBytes(flow.bytes_received)}</td>
                <td>{flow.packets_sent}</td>
                <td>{flow.packets_received}</td>
                <td>{formatTime(flow.start_time)}</td>
                <td>{formatTime(flow.last_activity)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  // Policies tab
  const PoliciesTab: React.FC = () => (
    <div className="policies-tab">
      <div className="tab-header">
        <h2>Network Policies</h2>
      </div>
      <div className="table-container">
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Description</th>
              <th>Priority</th>
              <th>Enabled</th>
              <th>Rules</th>
              <th>Created</th>
              <th>Updated</th>
            </tr>
          </thead>
          <tbody>
            {policies.map((policy) => (
              <tr key={policy.id}>
                <td>{policy.id}</td>
                <td>{policy.name}</td>
                <td>{policy.description}</td>
                <td>{policy.priority}</td>
                <td>
                  <span className={`policy-enabled policy-enabled-${policy.enabled}`}>
                    {policy.enabled ? 'Yes' : 'No'}
                  </span>
                </td>
                <td>
                  <div className="policy-rules">
                    {policy.rules.map((rule, index) => (
                      <span key={index} className="policy-rule">{rule}</span>
                    ))}
                  </div>
                </td>
                <td>{formatTime(policy.created_at)}</td>
                <td>{formatTime(policy.updated_at)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  return (
    <div className="network-panel">
      <div className="panel-header">
        <h1>Aetheris OS Network Management</h1>
        <div className="header-actions">
          <button 
            className="btn btn-secondary"
            onClick={loadData}
            disabled={loading}
          >
            {loading ? 'Refreshing...' : 'Refresh'}
          </button>
        </div>
      </div>

      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
          <button 
            className="btn btn-sm btn-secondary"
            onClick={() => setError(null)}
          >
            Dismiss
          </button>
        </div>
      )}

      <div className="tab-navigation">
        <button 
          className={`tab-button ${activeTab === 'overview' ? 'active' : ''}`}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button 
          className={`tab-button ${activeTab === 'namespaces' ? 'active' : ''}`}
          onClick={() => setActiveTab('namespaces')}
        >
          Namespaces
        </button>
        <button 
          className={`tab-button ${activeTab === 'sockets' ? 'active' : ''}`}
          onClick={() => setActiveTab('sockets')}
        >
          Sockets
        </button>
        <button 
          className={`tab-button ${activeTab === 'flows' ? 'active' : ''}`}
          onClick={() => setActiveTab('flows')}
        >
          Flows
        </button>
        <button 
          className={`tab-button ${activeTab === 'policies' ? 'active' : ''}`}
          onClick={() => setActiveTab('policies')}
        >
          Policies
        </button>
      </div>

      <div className="tab-content">
        {activeTab === 'overview' && <OverviewTab />}
        {activeTab === 'namespaces' && <NamespacesTab />}
        {activeTab === 'sockets' && <SocketsTab />}
        {activeTab === 'flows' && <FlowsTab />}
        {activeTab === 'policies' && <PoliciesTab />}
      </div>
    </div>
  );
};

export default NetworkPanel;

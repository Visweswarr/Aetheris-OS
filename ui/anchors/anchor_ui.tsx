import React, { useState, useEffect, useCallback } from 'react';
import './anchor_ui.css';

// Anchor types matching the CDDL schema
interface AnchorV1 {
  snap: Uint8Array;
  did: string;
  time: number;
  chain: string;
  version: string;
  metadata: AnchorMetadata;
  signature: Uint8Array;
  gas_used: number;
  block_number?: number;
  transaction_hash?: Uint8Array;
  status: AnchorStatus;
}

interface AnchorMetadata {
  description?: string;
  tags: string[];
  priority: AnchorPriority;
  batch_id?: string;
  expires_at?: number;
  custom_fields: Record<string, any>;
}

enum AnchorPriority {
  Low = 'low',
  Normal = 'normal',
  High = 'high',
  Critical = 'critical'
}

enum AnchorStatus {
  Pending = 'pending',
  Submitted = 'submitted',
  Confirmed = 'confirmed',
  Failed = 'failed',
  Expired = 'expired'
}

interface AnchorRequest {
  snapshot_cid: string;
  target_chain: string;
  priority: AnchorPriority;
  description?: string;
  tags: string[];
  gas_limit: number;
  max_fee_per_gas?: string;
  expires_at?: number;
}

interface AnchorResult {
  success: boolean;
  anchor_hash?: string;
  transaction_hash?: string;
  block_number?: number;
  gas_used?: number;
  error?: string;
  chain_id?: number;
  cost?: string;
}

interface ChainConfig {
  chain_id: number;
  name: string;
  rpc_url: string;
  contract_address: string;
  gas_limit: number;
  max_fee_per_gas: string;
  priority_fee: string;
  confirmations: number;
  timeout: number;
  enabled: boolean;
}

interface AnchorStats {
  total_anchors: number;
  successful_anchors: number;
  failed_anchors: number;
  total_gas_used: number;
  average_gas_per_anchor: number;
  total_cost: number;
  chains_used: string[];
  last_anchor_time?: number;
  pending_anchors: number;
}

// Main anchor explorer component
export const AnchorExplorer: React.FC = () => {
  const [anchors, setAnchors] = useState<AnchorV1[]>([]);
  const [selectedAnchor, setSelectedAnchor] = useState<AnchorV1 | null>(null);
  const [anchorRequests, setAnchorRequests] = useState<AnchorRequest[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterChain, setFilterChain] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<AnchorStatus | 'all'>('all');
  const [filterPriority, setFilterPriority] = useState<AnchorPriority | 'all'>('all');
  const [sortBy, setSortBy] = useState<'time' | 'priority' | 'chain' | 'status'>('time');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newAnchorRequest, setNewAnchorRequest] = useState<AnchorRequest>({
    snapshot_cid: '',
    target_chain: 'local',
    priority: AnchorPriority.Normal,
    description: '',
    tags: [],
    gas_limit: 100000,
    expires_at: undefined
  });
  const [stats, setStats] = useState<AnchorStats>({
    total_anchors: 0,
    successful_anchors: 0,
    failed_anchors: 0,
    total_gas_used: 0,
    average_gas_per_anchor: 0,
    total_cost: 0,
    chains_used: [],
    pending_anchors: 0
  });

  // Load mock anchors on component mount
  useEffect(() => {
    loadMockAnchors();
    loadMockStats();
  }, []);

  const loadMockAnchors = useCallback(() => {
    const mockAnchors: AnchorV1[] = [
      {
        snap: new Uint8Array(32).fill(1),
        did: 'did:aetheris:test:user1',
        time: Date.now() - 3600000,
        chain: 'local',
        version: '1.0',
        metadata: {
          description: 'Test NGFS snapshot anchor',
          tags: ['test', 'ngfs', 'snapshot'],
          priority: AnchorPriority.Normal,
          custom_fields: {}
        },
        signature: new Uint8Array(64).fill(2),
        gas_used: 45000,
        block_number: 12345,
        transaction_hash: new Uint8Array(32).fill(3),
        status: AnchorStatus.Confirmed
      },
      {
        snap: new Uint8Array(32).fill(4),
        did: 'did:aetheris:test:user2',
        time: Date.now() - 7200000,
        chain: 'local',
        version: '1.0',
        metadata: {
          description: 'High priority system snapshot',
          tags: ['system', 'critical', 'backup'],
          priority: AnchorPriority.High,
          custom_fields: {}
        },
        signature: new Uint8Array(64).fill(5),
        gas_used: 52000,
        block_number: 12340,
        transaction_hash: new Uint8Array(32).fill(6),
        status: AnchorStatus.Confirmed
      },
      {
        snap: new Uint8Array(32).fill(7),
        did: 'did:aetheris:test:user3',
        time: Date.now() - 1800000,
        chain: 'local',
        version: '1.0',
        metadata: {
          description: 'Pending user data snapshot',
          tags: ['user', 'data', 'pending'],
          priority: AnchorPriority.Low,
          custom_fields: {}
        },
        signature: new Uint8Array(64).fill(8),
        gas_used: 0,
        status: AnchorStatus.Pending
      }
    ];
    setAnchors(mockAnchors);
  }, []);

  const loadMockStats = useCallback(() => {
    const mockStats: AnchorStats = {
      total_anchors: 3,
      successful_anchors: 2,
      failed_anchors: 0,
      total_gas_used: 97000,
      average_gas_per_anchor: 48500,
      total_cost: 0.00194,
      chains_used: ['local'],
      last_anchor_time: Date.now() - 1800000,
      pending_anchors: 1
    };
    setStats(mockStats);
  }, []);

  const filteredAndSortedAnchors = useCallback(() => {
    let filtered = anchors.filter(anchor => {
      const matchesSearch = anchor.did.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           (anchor.metadata.description?.toLowerCase().includes(searchTerm.toLowerCase()) ?? false) ||
                           anchor.metadata.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
      
      const matchesChain = filterChain === 'all' || anchor.chain === filterChain;
      const matchesStatus = filterStatus === 'all' || anchor.status === filterStatus;
      const matchesPriority = filterPriority === 'all' || anchor.metadata.priority === filterPriority;
      
      return matchesSearch && matchesChain && matchesStatus && matchesPriority;
    });

    filtered.sort((a, b) => {
      let aValue: any, bValue: any;
      
      switch (sortBy) {
        case 'time':
          aValue = a.time;
          bValue = b.time;
          break;
        case 'priority':
          aValue = getPriorityValue(a.metadata.priority);
          bValue = getPriorityValue(b.metadata.priority);
          break;
        case 'chain':
          aValue = a.chain;
          bValue = b.chain;
          break;
        case 'status':
          aValue = getStatusValue(a.status);
          bValue = getStatusValue(b.status);
          break;
        default:
          aValue = a.time;
          bValue = b.time;
      }
      
      if (sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

    return filtered;
  }, [anchors, searchTerm, filterChain, filterStatus, filterPriority, sortBy, sortOrder]);

  const getPriorityValue = (priority: AnchorPriority): number => {
    switch (priority) {
      case AnchorPriority.Critical: return 4;
      case AnchorPriority.High: return 3;
      case AnchorPriority.Normal: return 2;
      case AnchorPriority.Low: return 1;
      default: return 0;
    }
  };

  const getStatusValue = (status: AnchorStatus): number => {
    switch (status) {
      case AnchorStatus.Confirmed: return 4;
      case AnchorStatus.Submitted: return 3;
      case AnchorStatus.Pending: return 2;
      case AnchorStatus.Failed: return 1;
      case AnchorStatus.Expired: return 0;
      default: return 0;
    }
  };

  const createAnchorRequest = useCallback(async () => {
    if (!newAnchorRequest.snapshot_cid || !newAnchorRequest.target_chain) {
      alert('Please fill in required fields');
      return;
    }

    // In production, this would submit to the blockchain
    const mockResult: AnchorResult = {
      success: true,
      anchor_hash: '0x' + Array(64).fill('a').join(''),
      transaction_hash: '0x' + Array(64).fill('b').join(''),
      block_number: Math.floor(Math.random() * 10000) + 12000,
      gas_used: Math.floor(Math.random() * 20000) + 40000,
      chain_id: 1337,
      cost: '0.0001 ETH'
    };

    if (mockResult.success) {
      // Add to anchors list
      const newAnchor: AnchorV1 = {
        snap: new Uint8Array(32).fill(9),
        did: 'did:aetheris:current:user',
        time: Date.now(),
        chain: newAnchorRequest.target_chain,
        version: '1.0',
        metadata: {
          description: newAnchorRequest.description,
          tags: newAnchorRequest.tags,
          priority: newAnchorRequest.priority,
          expires_at: newAnchorRequest.expires_at,
          custom_fields: {}
        },
        signature: new Uint8Array(64).fill(10),
        gas_used: mockResult.gas_used!,
        block_number: mockResult.block_number,
        transaction_hash: new Uint8Array(32).fill(11),
        status: AnchorStatus.Confirmed
      };

      setAnchors(prev => [newAnchor, ...prev]);
      setShowCreateForm(false);
      setNewAnchorRequest({
        snapshot_cid: '',
        target_chain: 'local',
        priority: AnchorPriority.Normal,
        description: '',
        tags: [],
        gas_limit: 100000,
        expires_at: undefined
      });

      // Update stats
      setStats(prev => ({
        ...prev,
        total_anchors: prev.total_anchors + 1,
        successful_anchors: prev.successful_anchors + 1,
        total_gas_used: prev.total_gas_used + mockResult.gas_used!,
        average_gas_per_anchor: (prev.total_gas_used + mockResult.gas_used!) / (prev.successful_anchors + 1),
        last_anchor_time: Date.now()
      }));
    }
  }, [newAnchorRequest]);

  const formatBytes = (bytes: Uint8Array, maxLength: number = 16): string => {
    const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    return '0x' + hex.substring(0, maxLength) + (hex.length > maxLength ? '...' : '');
  };

  const formatTimestamp = (timestamp: number): string => {
    return new Date(timestamp).toLocaleString();
  };

  const formatGas = (gas: number): string => {
    return gas.toLocaleString();
  };

  const getPriorityColor = (priority: AnchorPriority): string => {
    switch (priority) {
      case AnchorPriority.Critical: return '#dc3545';
      case AnchorPriority.High: return '#fd7e14';
      case AnchorPriority.Normal: return '#6c757d';
      case AnchorPriority.Low: return '#28a745';
      default: return '#6c757d';
    }
  };

  const getStatusColor = (status: AnchorStatus): string => {
    switch (status) {
      case AnchorStatus.Confirmed: return '#28a745';
      case AnchorStatus.Submitted: return '#17a2b8';
      case AnchorStatus.Pending: return '#ffc107';
      case AnchorStatus.Failed: return '#dc3545';
      case AnchorStatus.Expired: return '#6c757d';
      default: return '#6c757d';
    }
  };

  const getStatusIcon = (status: AnchorStatus): string => {
    switch (status) {
      case AnchorStatus.Confirmed: return '✓';
      case AnchorStatus.Submitted: return '→';
      case AnchorStatus.Pending: return '⏳';
      case AnchorStatus.Failed: return '✗';
      case AnchorStatus.Expired: return '⏰';
      default: return '?';
    }
  };

  return (
    <div className="anchor-explorer">
      <div className="anchor-header">
        <h1>NGFS Snapshot Anchors</h1>
        <p>On-chain audit anchoring for immutable snapshot verification</p>
        
        <div className="anchor-stats">
          <div className="stat-item">
            <span className="stat-value">{stats.total_anchors}</span>
            <span className="stat-label">Total Anchors</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.successful_anchors}</span>
            <span className="stat-label">Confirmed</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.pending_anchors}</span>
            <span className="stat-label">Pending</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stats.total_gas_used.toLocaleString()}</span>
            <span className="stat-label">Total Gas</span>
          </div>
        </div>
      </div>

      <div className="anchor-controls">
        <div className="search-controls">
          <input
            type="text"
            placeholder="Search anchors..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
          
          <select
            value={filterChain}
            onChange={(e) => setFilterChain(e.target.value)}
            className="filter-select"
          >
            <option value="all">All Chains</option>
            <option value="local">Local</option>
            <option value="testnet">Testnet</option>
            <option value="mainnet">Mainnet</option>
          </select>
          
          <select
            value={filterStatus}
            onChange={(e) => setFilterStatus(e.target.value as AnchorStatus | 'all')}
            className="filter-select"
          >
            <option value="all">All Status</option>
            <option value={AnchorStatus.Pending}>Pending</option>
            <option value={AnchorStatus.Submitted}>Submitted</option>
            <option value={AnchorStatus.Confirmed}>Confirmed</option>
            <option value={AnchorStatus.Failed}>Failed</option>
            <option value={AnchorStatus.Expired}>Expired</option>
          </select>
          
          <select
            value={filterPriority}
            onChange={(e) => setFilterPriority(e.target.value as AnchorPriority | 'all')}
            className="filter-select"
          >
            <option value="all">All Priorities</option>
            <option value={AnchorPriority.Low}>Low</option>
            <option value={AnchorPriority.Normal}>Normal</option>
            <option value={AnchorPriority.High}>High</option>
            <option value={AnchorPriority.Critical}>Critical</option>
          </select>
        </div>
        
        <div className="sort-controls">
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="sort-select"
          >
            <option value="time">Time</option>
            <option value="priority">Priority</option>
            <option value="chain">Chain</option>
            <option value="status">Status</option>
          </select>
          
          <button
            onClick={() => setSortOrder(prev => prev === 'asc' ? 'desc' : 'asc')}
            className="sort-button"
          >
            {sortOrder === 'asc' ? '↑' : '↓'}
          </button>
          
          <button
            onClick={() => setShowCreateForm(true)}
            className="create-button"
          >
            + New Anchor
          </button>
        </div>
      </div>

      {showCreateForm && (
        <div className="create-form-overlay">
          <div className="create-form">
            <h3>Create New Anchor</h3>
            
            <div className="form-group">
              <label>Snapshot CID:</label>
              <input
                type="text"
                value={newAnchorRequest.snapshot_cid}
                onChange={(e) => setNewAnchorRequest(prev => ({ ...prev, snapshot_cid: e.target.value }))}
                placeholder="0x..."
                required
              />
            </div>
            
            <div className="form-group">
              <label>Target Chain:</label>
              <select
                value={newAnchorRequest.target_chain}
                onChange={(e) => setNewAnchorRequest(prev => ({ ...prev, target_chain: e.target.value }))}
              >
                <option value="local">Local</option>
                <option value="testnet">Testnet</option>
                <option value="mainnet">Mainnet</option>
              </select>
            </div>
            
            <div className="form-group">
              <label>Priority:</label>
              <select
                value={newAnchorRequest.priority}
                onChange={(e) => setNewAnchorRequest(prev => ({ ...prev, priority: e.target.value as AnchorPriority }))}
              >
                <option value={AnchorPriority.Low}>Low</option>
                <option value={AnchorPriority.Normal}>Normal</option>
                <option value={AnchorPriority.High}>High</option>
                <option value={AnchorPriority.Critical}>Critical</option>
              </select>
            </div>
            
            <div className="form-group">
              <label>Description:</label>
              <input
                type="text"
                value={newAnchorRequest.description}
                onChange={(e) => setNewAnchorRequest(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Optional description"
              />
            </div>
            
            <div className="form-group">
              <label>Tags (comma-separated):</label>
              <input
                type="text"
                value={newAnchorRequest.tags.join(', ')}
                onChange={(e) => setNewAnchorRequest(prev => ({ 
                  ...prev, 
                  tags: e.target.value.split(',').map(t => t.trim()).filter(t => t)
                }))}
                placeholder="tag1, tag2, tag3"
              />
            </div>
            
            <div className="form-group">
              <label>Gas Limit:</label>
              <input
                type="number"
                value={newAnchorRequest.gas_limit}
                onChange={(e) => setNewAnchorRequest(prev => ({ ...prev, gas_limit: parseInt(e.target.value) }))}
                min="21000"
                max="1000000"
              />
            </div>
            
            <div className="form-group">
              <label>Expires At (optional):</label>
              <input
                type="datetime-local"
                onChange={(e) => setNewAnchorRequest(prev => ({ 
                  ...prev, 
                  expires_at: e.target.value ? new Date(e.target.value).getTime() / 1000 : undefined
                }))}
              />
            </div>
            
            <div className="form-actions">
              <button onClick={createAnchorRequest} className="submit-button">
                Create Anchor
              </button>
              <button onClick={() => setShowCreateForm(false)} className="cancel-button">
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="anchors-list">
        {filteredAndSortedAnchors().map((anchor, index) => (
          <div
            key={index}
            className={`anchor-item ${selectedAnchor === anchor ? 'selected' : ''}`}
            onClick={() => setSelectedAnchor(anchor === selectedAnchor ? null : anchor)}
          >
            <div className="anchor-header-row">
              <div className="anchor-status">
                <span 
                  className="status-icon"
                  style={{ color: getStatusColor(anchor.status) }}
                >
                  {getStatusIcon(anchor.status)}
                </span>
                <span className="status-text">{anchor.status}</span>
              </div>
              
              <div className="anchor-priority">
                <span 
                  className="priority-badge"
                  style={{ backgroundColor: getPriorityColor(anchor.metadata.priority) }}
                >
                  {anchor.metadata.priority}
                </span>
              </div>
            </div>
            
            <div className="anchor-content">
              <div className="anchor-main">
                <div className="snapshot-cid">
                  <strong>Snapshot:</strong> {formatBytes(anchor.snap)}
                </div>
                <div className="anchor-did">
                  <strong>DID:</strong> {anchor.did}
                </div>
                <div className="anchor-chain">
                  <strong>Chain:</strong> {anchor.chain}
                </div>
                {anchor.metadata.description && (
                  <div className="anchor-description">
                    {anchor.metadata.description}
                  </div>
                )}
                {anchor.metadata.tags.length > 0 && (
                  <div className="anchor-tags">
                    {anchor.metadata.tags.map((tag, tagIndex) => (
                      <span key={tagIndex} className="tag">{tag}</span>
                    ))}
                  </div>
                )}
              </div>
              
              <div className="anchor-details">
                <div className="anchor-time">
                  <strong>Time:</strong> {formatTimestamp(anchor.time)}
                </div>
                {anchor.block_number && (
                  <div className="anchor-block">
                    <strong>Block:</strong> {anchor.block_number.toLocaleString()}
                  </div>
                )}
                {anchor.transaction_hash && (
                  <div className="anchor-tx">
                    <strong>TX:</strong> {formatBytes(anchor.transaction_hash)}
                  </div>
                )}
                {anchor.gas_used > 0 && (
                  <div className="anchor-gas">
                    <strong>Gas:</strong> {formatGas(anchor.gas_used)}
                  </div>
                )}
              </div>
            </div>
            
            {selectedAnchor === anchor && (
              <div className="anchor-expanded">
                <div className="expanded-section">
                  <h4>Raw Data</h4>
                  <div className="raw-data">
                    <div><strong>Signature:</strong> {formatBytes(anchor.signature, 32)}</div>
                    <div><strong>Version:</strong> {anchor.version}</div>
                    {anchor.metadata.batch_id && (
                      <div><strong>Batch ID:</strong> {anchor.metadata.batch_id}</div>
                    )}
                    {anchor.metadata.expires_at && (
                      <div><strong>Expires:</strong> {formatTimestamp(anchor.metadata.expires_at * 1000)}</div>
                    )}
                  </div>
                </div>
                
                <div className="expanded-section">
                  <h4>Custom Fields</h4>
                  {Object.keys(anchor.metadata.custom_fields).length > 0 ? (
                    <div className="custom-fields">
                      {Object.entries(anchor.metadata.custom_fields).map(([key, value]) => (
                        <div key={key}>
                          <strong>{key}:</strong> {JSON.stringify(value)}
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="no-custom-fields">No custom fields</div>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
        
        {filteredAndSortedAnchors().length === 0 && (
          <div className="no-anchors">
            <p>No anchors found matching the current filters.</p>
            <p>Try adjusting your search criteria or create a new anchor.</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default AnchorExplorer;

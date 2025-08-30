import React, { useState, useEffect, useCallback } from 'react';
import './vault_ui.css';

// Types for vault entries and capabilities
export interface VaultEntry {
  id: string;
  kind: EntryKind;
  subject_did: string;
  size: number;
  created_at: string;
  updated_at: string;
  tags: string[];
  meta: Record<string, any>;
  revoked: boolean;
}

export enum EntryKind {
  Key = 'key',
  Document = 'document',
  Credential = 'credential',
  Secret = 'secret',
  Backup = 'backup'
}

export interface Capability {
  operation: string;
  resource: string;
  expires_at?: string;
}

export interface CapTokenV2 {
  version: number;
  issuer_did: string;
  subject_did: string;
  capabilities: Capability[];
  issued_at: string;
  expires_at?: string;
  signature: string;
  nonce: string;
}

export interface VaultOperation {
  operation: string;
  entry_id?: string;
  cap_token: CapTokenV2;
  timestamp: string;
  request_id: string;
}

// Props for the VaultUI component
export interface VaultUIProps {
  entries?: VaultEntry[];
  capToken?: CapTokenV2;
  onOperation?: (operation: VaultOperation) => void;
  onCapTokenRequest?: () => void;
  className?: string;
}

// Main VaultUI component
export const VaultUI: React.FC<VaultUIProps> = ({
  entries = [],
  capToken,
  onOperation,
  onCapTokenRequest,
  className = ''
}) => {
  const [selectedEntry, setSelectedEntry] = useState<VaultEntry | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterKind, setFilterKind] = useState<EntryKind | 'all'>('all');
  const [sortBy, setSortBy] = useState<'created_at' | 'updated_at' | 'id' | 'size'>('created_at');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [loading, setLoading] = useState(false);

  // Filter and sort entries
  const filteredAndSortedEntries = useCallback(() => {
    let filtered = entries.filter(entry => {
      const matchesSearch = entry.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           entry.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
      const matchesKind = filterKind === 'all' || entry.kind === filterKind;
      return matchesSearch && matchesKind;
    });

    filtered.sort((a, b) => {
      let aValue: any = a[sortBy];
      let bValue: any = b[sortBy];

      if (sortBy === 'created_at' || sortBy === 'updated_at') {
        aValue = new Date(aValue).getTime();
        bValue = new Date(bValue).getTime();
      }

      if (sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

    return filtered;
  }, [entries, searchQuery, filterKind, sortBy, sortOrder]);

  // Handle entry selection
  const handleEntrySelect = (entry: VaultEntry) => {
    setSelectedEntry(entry);
  };

  // Handle vault operations
  const handleVaultOperation = useCallback((operation: string, entryId?: string) => {
    if (!capToken) {
      if (onCapTokenRequest) {
        onCapTokenRequest();
      }
      return;
    }

    if (onOperation) {
      const vaultOp: VaultOperation = {
        operation,
        entry_id: entryId,
        cap_token: capToken,
        timestamp: new Date().toISOString(),
        request_id: `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
      };
      onOperation(vaultOp);
    }
  }, [capToken, onOperation, onCapTokenRequest]);

  // Check if user has capability for operation
  const hasCapability = useCallback((operation: string, resource: string = '*') => {
    if (!capToken) return false;
    
    return capToken.capabilities.some(cap => {
      if (cap.operation !== operation) return false;
      
      if (cap.resource === '*' || cap.resource === resource) return true;
      
      // Simple wildcard matching
      if (cap.resource.endsWith('*')) {
        const prefix = cap.resource.slice(0, -1);
        return resource.startsWith(prefix);
      }
      
      return false;
    });
  }, [capToken]);

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Format date
  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  // Get entry kind icon
  const getEntryKindIcon = (kind: EntryKind): string => {
    switch (kind) {
      case EntryKind.Key: return '🔑';
      case EntryKind.Document: return '📄';
      case EntryKind.Credential: return '🆔';
      case EntryKind.Secret: return '🤐';
      case EntryKind.Backup: return '💾';
      default: return '📁';
    }
  };

  // Get entry kind color
  const getEntryKindColor = (kind: EntryKind): string => {
    switch (kind) {
      case EntryKind.Key: return '#ff6b6b';
      case EntryKind.Document: return '#4ecdc4';
      case EntryKind.Credential: return '#45b7d1';
      case EntryKind.Secret: return '#96ceb4';
      case EntryKind.Backup: return '#feca57';
      default: return '#c44569';
    }
  };

  return (
    <div className={`vault-ui ${className}`}>
      {/* Header */}
      <div className="vault-header">
        <h1>🔐 NGFS Personal Data Vault</h1>
        <div className="vault-status">
          {capToken ? (
            <span className="status-connected">
              ✅ Connected as {capToken.subject_did}
            </span>
          ) : (
            <span className="status-disconnected">
              ❌ No capability token provided
            </span>
          )}
        </div>
      </div>

      {/* Controls */}
      <div className="vault-controls">
        <div className="search-controls">
          <input
            type="text"
            placeholder="Search entries..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
          <select
            value={filterKind}
            onChange={(e) => setFilterKind(e.target.value as EntryKind | 'all')}
            className="filter-select"
          >
            <option value="all">All Types</option>
            {Object.values(EntryKind).map(kind => (
              <option key={kind} value={kind}>
                {getEntryKindIcon(kind)} {kind.charAt(0).toUpperCase() + kind.slice(1)}
              </option>
            ))}
          </select>
        </div>

        <div className="sort-controls">
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="sort-select"
          >
            <option value="created_at">Created</option>
            <option value="updated_at">Updated</option>
            <option value="id">ID</option>
            <option value="size">Size</option>
          </select>
          <button
            onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')}
            className="sort-order-btn"
          >
            {sortOrder === 'asc' ? '↑' : '↓'}
          </button>
        </div>

        <div className="action-controls">
          {hasCapability('write', '*') && (
            <button
              onClick={() => setShowAddForm(true)}
              className="btn btn-primary"
            >
              ➕ Add Entry
            </button>
          )}
          {!capToken && (
            <button
              onClick={() => onCapTokenRequest?.()}
              className="btn btn-secondary"
            >
              🔑 Provide Capability Token
            </button>
          )}
        </div>
      </div>

      {/* Main Content */}
      <div className="vault-content">
        {/* Entries List */}
        <div className="entries-list">
          <div className="entries-header">
            <h2>Vault Entries ({filteredAndSortedEntries().length})</h2>
          </div>
          
          {filteredAndSortedEntries().length === 0 ? (
            <div className="no-entries">
              {entries.length === 0 ? (
                <p>No entries in vault</p>
              ) : (
                <p>No entries match your search criteria</p>
              )}
            </div>
          ) : (
            <div className="entries-grid">
              {filteredAndSortedEntries().map(entry => (
                <div
                  key={entry.id}
                  className={`entry-card ${selectedEntry?.id === entry.id ? 'selected' : ''} ${entry.revoked ? 'revoked' : ''}`}
                  onClick={() => handleEntrySelect(entry)}
                >
                  <div className="entry-header">
                    <span className="entry-kind-icon" style={{ color: getEntryKindColor(entry.kind) }}>
                      {getEntryKindIcon(entry.kind)}
                    </span>
                    <span className="entry-id">{entry.id}</span>
                    {entry.revoked && <span className="revoked-badge">REVOKED</span>}
                  </div>
                  
                  <div className="entry-details">
                    <div className="entry-meta">
                      <span className="entry-size">{formatFileSize(entry.size)}</span>
                      <span className="entry-created">{formatDate(entry.created_at)}</span>
                    </div>
                    
                    {entry.tags.length > 0 && (
                      <div className="entry-tags">
                        {entry.tags.slice(0, 3).map(tag => (
                          <span key={tag} className="tag">{tag}</span>
                        ))}
                        {entry.tags.length > 3 && (
                          <span className="tag-more">+{entry.tags.length - 3}</span>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Entry Details */}
        {selectedEntry && (
          <div className="entry-details-panel">
            <div className="entry-details-header">
              <h3>Entry Details</h3>
              <button
                onClick={() => setSelectedEntry(null)}
                className="close-btn"
              >
                ×
              </button>
            </div>
            
            <div className="entry-details-content">
              <div className="detail-row">
                <label>ID:</label>
                <span>{selectedEntry.id}</span>
              </div>
              
              <div className="detail-row">
                <label>Type:</label>
                <span className="entry-type">
                  {getEntryKindIcon(selectedEntry.kind)} {selectedEntry.kind}
                </span>
              </div>
              
              <div className="detail-row">
                <label>Size:</label>
                <span>{formatFileSize(selectedEntry.size)}</span>
              </div>
              
              <div className="detail-row">
                <label>Subject DID:</label>
                <span className="did">{selectedEntry.subject_did}</span>
              </div>
              
              <div className="detail-row">
                <label>Created:</label>
                <span>{formatDate(selectedEntry.created_at)}</span>
              </div>
              
              <div className="detail-row">
                <label>Updated:</label>
                <span>{formatDate(selectedEntry.updated_at)}</span>
              </div>
              
              {selectedEntry.tags.length > 0 && (
                <div className="detail-row">
                  <label>Tags:</label>
                  <div className="tags-list">
                    {selectedEntry.tags.map(tag => (
                      <span key={tag} className="tag">{tag}</span>
                    ))}
                  </div>
                </div>
              )}
              
              {Object.keys(selectedEntry.meta).length > 0 && (
                <div className="detail-row">
                  <label>Metadata:</label>
                  <div className="metadata-list">
                    {Object.entries(selectedEntry.meta).map(([key, value]) => (
                      <div key={key} className="metadata-item">
                        <span className="metadata-key">{key}:</span>
                        <span className="metadata-value">{String(value)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
            
            <div className="entry-actions">
              {hasCapability('read', selectedEntry.id) && (
                <button
                  onClick={() => handleVaultOperation('show', selectedEntry.id)}
                  className="btn btn-primary"
                  disabled={loading}
                >
                  👁️ Show Content
                </button>
              )}
              
              {hasCapability('delete', selectedEntry.id) && !selectedEntry.revoked && (
                <button
                  onClick={() => handleVaultOperation('delete', selectedEntry.id)}
                  className="btn btn-danger"
                  disabled={loading}
                >
                  🗑️ Delete Entry
                </button>
              )}
              
              {!hasCapability('read', selectedEntry.id) && (
                <span className="insufficient-capabilities">
                  ❌ Insufficient capabilities for this entry
                </span>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Add Entry Form */}
      {showAddForm && (
        <div className="modal-overlay">
          <div className="modal">
            <div className="modal-header">
              <h3>Add New Vault Entry</h3>
              <button
                onClick={() => setShowAddForm(false)}
                className="close-btn"
              >
                ×
              </button>
            </div>
            
            <div className="modal-content">
              <AddEntryForm
                onSubmit={(entryData) => {
                  handleVaultOperation('add');
                  setShowAddForm(false);
                }}
                onCancel={() => setShowAddForm(false)}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Add Entry Form Component
interface AddEntryFormProps {
  onSubmit: (entryData: any) => void;
  onCancel: () => void;
}

const AddEntryForm: React.FC<AddEntryFormProps> = ({ onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    id: '',
    kind: EntryKind.Document,
    tags: '',
    meta: ''
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const entryData = {
      ...formData,
      tags: formData.tags.split(',').map(tag => tag.trim()).filter(tag => tag),
      meta: formData.meta ? JSON.parse(formData.meta) : {}
    };
    
    onSubmit(entryData);
  };

  return (
    <form onSubmit={handleSubmit} className="add-entry-form">
      <div className="form-group">
        <label htmlFor="entry-id">Entry ID:</label>
        <input
          id="entry-id"
          type="text"
          value={formData.id}
          onChange={(e) => setFormData({ ...formData, id: e.target.value })}
          required
          placeholder="Enter unique entry ID"
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="entry-kind">Entry Type:</label>
        <select
          id="entry-kind"
          value={formData.kind}
          onChange={(e) => setFormData({ ...formData, kind: e.target.value as EntryKind })}
        >
          {Object.values(EntryKind).map(kind => (
            <option key={kind} value={kind}>
              {kind.charAt(0).toUpperCase() + kind.slice(1)}
            </option>
          ))}
        </select>
      </div>
      
      <div className="form-group">
        <label htmlFor="entry-tags">Tags (comma-separated):</label>
        <input
          id="entry-tags"
          type="text"
          value={formData.tags}
          onChange={(e) => setFormData({ ...formData, tags: e.target.value })}
          placeholder="tag1, tag2, tag3"
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="entry-meta">Metadata (JSON):</label>
        <textarea
          id="entry-meta"
          value={formData.meta}
          onChange={(e) => setFormData({ ...formData, meta: e.target.value })}
          placeholder='{"key": "value"}'
          rows={3}
        />
      </div>
      
      <div className="form-actions">
        <button type="submit" className="btn btn-primary">
          Add Entry
        </button>
        <button type="button" onClick={onCancel} className="btn btn-secondary">
          Cancel
        </button>
      </div>
    </form>
  );
};

// Export the component
export default VaultUI;

/**
 * DAO Bridge - TypeScript SDK for Aetheris OS DAO operations
 * 
 * This module provides a TypeScript interface for interacting with the DAO system,
 * including proposal creation, voting, execution, and governance management.
 */

import { EventEmitter } from 'events';

// Type definitions
export type ProposalType = 'add-skill' | 'update-policy' | 'change-governance' | 'execute-code' | 'transfer-funds' | 'custom';
export type ProposalStatus = 'draft' | 'active' | 'voting-ended' | 'passed' | 'failed' | 'executed' | 'cancelled';
export type VoteChoice = 'yes' | 'no' | 'abstain';
export type ExecutionMethod = 'contract-call' | 'system-call' | 'policy-update' | 'config-change' | 'custom';
export type VotingPowerMethod = 'one-vote-per-address' | 'token-balance' | 'reputation-score' | 'stake-amount' | 'custom';
export type MemberStatus = 'active' | 'suspended' | 'banned' | 'inactive';

export interface Proposal {
  id: string;
  title: string;
  description: string;
  proposalType: ProposalType;
  startTime: Date;
  endTime: Date;
  proposer: string;
  proposerDID: string;
  status: ProposalStatus;
  executionData?: ExecutionData;
  createdAt: Date;
  updatedAt: Date;
}

export interface ExecutionData {
  target: string;
  parameters: Record<string, any>;
  method: ExecutionMethod;
  gasLimit?: number;
  value?: number;
}

export interface Vote {
  id: string;
  proposalId: string;
  voter: string;
  voterDID: string;
  choice: VoteChoice;
  weight: number;
  reason?: string;
  signature: string;
  timestamp: Date;
}

export interface VoteResult {
  proposalId: string;
  totalVotes: number;
  yesVotes: number;
  noVotes: number;
  abstainVotes: number;
  totalPower: number;
  quorumAchieved: boolean;
  majorityAchieved: boolean;
  result: 'passed' | 'failed' | 'no-quorum' | 'no-majority';
  calculatedAt: Date;
}

export interface ExecutionResult {
  proposalId: string;
  success: boolean;
  output?: string;
  error?: string;
  gasUsed?: number;
  txHash?: string;
  executedAt: Date;
  executor: string;
}

export interface DAOMember {
  address: string;
  did: string;
  reputation: number;
  stake: number;
  votingPower: number;
  status: MemberStatus;
  joinedAt: Date;
  lastActivity: Date;
}

export interface GovernanceParams {
  minVotingPeriod: number;
  maxVotingPeriod: number;
  minVotesRequired: number;
  majorityThreshold: number;
  quorumThreshold: number;
  executionDelay: number;
  proposalDeposit: number;
  votingPowerMethod: VotingPowerMethod;
}

export interface DAOStats {
  totalProposals: number;
  activeProposals: number;
  passedProposals: number;
  failedProposals: number;
  executedProposals: number;
  totalVotes: number;
  totalMembers: number;
  activeMembers: number;
  totalVotingPower: number;
  lastUpdated: Date;
}

export interface CreateProposalOptions {
  title: string;
  description: string;
  proposalType: ProposalType;
  votingPeriod?: number;
  proposer: string;
  proposerDID: string;
  executionData?: ExecutionData;
}

export interface VoteOptions {
  proposalId: string;
  choice: VoteChoice;
  voter: string;
  voterDID: string;
  weight?: number;
  reason?: string;
}

export interface ExecuteOptions {
  proposalId: string;
  executor: string;
}

export interface ListProposalsOptions {
  status?: ProposalStatus;
  proposer?: string;
  limit?: number;
  offset?: number;
}

/**
 * AetherisDAO - Main DAO SDK class
 */
export class AetherisDAO extends EventEmitter {
  private baseUrl: string;
  private apiKey?: string;
  private timeout: number;

  constructor(options: {
    baseUrl?: string;
    apiKey?: string;
    timeout?: number;
  } = {}) {
    super();
    this.baseUrl = options.baseUrl || 'http://localhost:8080';
    this.apiKey = options.apiKey;
    this.timeout = options.timeout || 30000;
  }

  /**
   * Create a new proposal
   */
  async createProposal(options: CreateProposalOptions): Promise<Proposal> {
    const response = await this.makeRequest('POST', '/api/dao/proposals', {
      title: options.title,
      description: options.description,
      proposalType: options.proposalType,
      votingPeriod: options.votingPeriod || 604800, // 7 days default
      proposer: options.proposer,
      proposerDID: options.proposerDID,
      executionData: options.executionData,
    });

    return this.parseProposal(response);
  }

  /**
   * Get a proposal by ID
   */
  async getProposal(proposalId: string): Promise<Proposal> {
    const response = await this.makeRequest('GET', `/api/dao/proposals/${proposalId}`);
    return this.parseProposal(response);
  }

  /**
   * List proposals with optional filters
   */
  async listProposals(options: ListProposalsOptions = {}): Promise<{
    proposals: Proposal[];
    totalCount: number;
    hasMore: boolean;
  }> {
    const params = new URLSearchParams();
    if (options.status) params.append('status', options.status);
    if (options.proposer) params.append('proposer', options.proposer);
    if (options.limit) params.append('limit', options.limit.toString());
    if (options.offset) params.append('offset', options.offset.toString());

    const response = await this.makeRequest('GET', `/api/dao/proposals?${params}`);
    
    return {
      proposals: response.proposals.map((p: any) => this.parseProposal(p)),
      totalCount: response.totalCount,
      hasMore: response.hasMore,
    };
  }

  /**
   * Activate a proposal (start voting)
   */
  async activateProposal(proposalId: string): Promise<void> {
    await this.makeRequest('POST', `/api/dao/proposals/${proposalId}/activate`);
    this.emit('proposalActivated', { proposalId });
  }

  /**
   * Cancel a proposal
   */
  async cancelProposal(proposalId: string, canceller: string): Promise<void> {
    await this.makeRequest('POST', `/api/dao/proposals/${proposalId}/cancel`, { canceller });
    this.emit('proposalCancelled', { proposalId, canceller });
  }

  /**
   * Cast a vote on a proposal
   */
  async vote(options: VoteOptions): Promise<Vote> {
    const response = await this.makeRequest('POST', '/api/dao/votes', {
      proposalId: options.proposalId,
      choice: options.choice,
      voter: options.voter,
      voterDID: options.voterDID,
      weight: options.weight || 1,
      reason: options.reason,
    });

    const vote = this.parseVote(response);
    this.emit('voteCast', vote);
    return vote;
  }

  /**
   * Get votes for a proposal
   */
  async getVotes(proposalId: string): Promise<Vote[]> {
    const response = await this.makeRequest('GET', `/api/dao/proposals/${proposalId}/votes`);
    return response.votes.map((v: any) => this.parseVote(v));
  }

  /**
   * Get vote result for a proposal
   */
  async getVoteResult(proposalId: string): Promise<VoteResult> {
    const response = await this.makeRequest('GET', `/api/dao/proposals/${proposalId}/result`);
    return this.parseVoteResult(response);
  }

  /**
   * Execute a proposal
   */
  async executeProposal(options: ExecuteOptions): Promise<ExecutionResult> {
    const response = await this.makeRequest('POST', '/api/dao/execute', {
      proposalId: options.proposalId,
      executor: options.executor,
    });

    const result = this.parseExecutionResult(response);
    this.emit('proposalExecuted', result);
    return result;
  }

  /**
   * Get execution result for a proposal
   */
  async getExecutionResult(proposalId: string): Promise<ExecutionResult | null> {
    try {
      const response = await this.makeRequest('GET', `/api/dao/proposals/${proposalId}/execution`);
      return this.parseExecutionResult(response);
    } catch (error) {
      if (error.status === 404) {
        return null;
      }
      throw error;
    }
  }

  /**
   * Get DAO statistics
   */
  async getStats(): Promise<DAOStats> {
    const response = await this.makeRequest('GET', '/api/dao/stats');
    return this.parseDAOStats(response);
  }

  /**
   * Get governance parameters
   */
  async getGovernanceParams(): Promise<GovernanceParams> {
    const response = await this.makeRequest('GET', '/api/dao/governance/params');
    return this.parseGovernanceParams(response);
  }

  /**
   * Update governance parameters (requires admin privileges)
   */
  async updateGovernanceParams(params: Partial<GovernanceParams>, updater: string): Promise<void> {
    await this.makeRequest('PUT', '/api/dao/governance/params', {
      ...params,
      updater,
    });
    this.emit('governanceUpdated', { params, updater });
  }

  /**
   * Get DAO members
   */
  async getMembers(): Promise<DAOMember[]> {
    const response = await this.makeRequest('GET', '/api/dao/members');
    return response.members.map((m: any) => this.parseDAOMember(m));
  }

  /**
   * Get member by address
   */
  async getMember(address: string): Promise<DAOMember | null> {
    try {
      const response = await this.makeRequest('GET', `/api/dao/members/${address}`);
      return this.parseDAOMember(response);
    } catch (error) {
      if (error.status === 404) {
        return null;
      }
      throw error;
    }
  }

  /**
   * Add a new member (requires admin privileges)
   */
  async addMember(member: Omit<DAOMember, 'joinedAt' | 'lastActivity'>): Promise<void> {
    await this.makeRequest('POST', '/api/dao/members', member);
    this.emit('memberAdded', member);
  }

  /**
   * Update member information (requires admin privileges)
   */
  async updateMember(address: string, updates: Partial<DAOMember>): Promise<void> {
    await this.makeRequest('PUT', `/api/dao/members/${address}`, updates);
    this.emit('memberUpdated', { address, updates });
  }

  /**
   * Remove a member (requires admin privileges)
   */
  async removeMember(address: string): Promise<void> {
    await this.makeRequest('DELETE', `/api/dao/members/${address}`);
    this.emit('memberRemoved', { address });
  }

  /**
   * Calculate voting power for a member
   */
  async calculateVotingPower(address: string): Promise<number> {
    const response = await this.makeRequest('GET', `/api/dao/members/${address}/voting-power`);
    return response.votingPower;
  }

  /**
   * Check if an address is authorized for a specific action
   */
  async isAuthorized(address: string, action: string): Promise<boolean> {
    const response = await this.makeRequest('GET', `/api/dao/authorization/${address}/${action}`);
    return response.authorized;
  }

  /**
   * Get governance history
   */
  async getGovernanceHistory(): Promise<any[]> {
    const response = await this.makeRequest('GET', '/api/dao/governance/history');
    return response.history;
  }

  /**
   * Get governance metrics
   */
  async getGovernanceMetrics(): Promise<any> {
    const response = await this.makeRequest('GET', '/api/dao/governance/metrics');
    return response.metrics;
  }

  /**
   * Subscribe to real-time updates
   */
  async subscribe(): Promise<void> {
    // This would implement WebSocket or Server-Sent Events
    // For now, we'll emit a mock event
    this.emit('subscribed');
  }

  /**
   * Unsubscribe from real-time updates
   */
  async unsubscribe(): Promise<void> {
    this.emit('unsubscribed');
  }

  // Private helper methods

  private async makeRequest(method: string, path: string, data?: any): Promise<any> {
    const url = `${this.baseUrl}${path}`;
    const options: RequestInit = {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...(this.apiKey && { 'Authorization': `Bearer ${this.apiKey}` }),
      },
      ...(data && { body: JSON.stringify(data) }),
    };

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, { ...options, signal: controller.signal });
      clearTimeout(timeoutId);

      if (!response.ok) {
        const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
        (error as any).status = response.status;
        throw error;
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      if (error.name === 'AbortError') {
        throw new Error('Request timeout');
      }
      throw error;
    }
  }

  private parseProposal(data: any): Proposal {
    return {
      id: data.id,
      title: data.title,
      description: data.description,
      proposalType: data.proposalType,
      startTime: new Date(data.startTime),
      endTime: new Date(data.endTime),
      proposer: data.proposer,
      proposerDID: data.proposerDID,
      status: data.status,
      executionData: data.executionData,
      createdAt: new Date(data.createdAt),
      updatedAt: new Date(data.updatedAt),
    };
  }

  private parseVote(data: any): Vote {
    return {
      id: data.id,
      proposalId: data.proposalId,
      voter: data.voter,
      voterDID: data.voterDID,
      choice: data.choice,
      weight: data.weight,
      reason: data.reason,
      signature: data.signature,
      timestamp: new Date(data.timestamp),
    };
  }

  private parseVoteResult(data: any): VoteResult {
    return {
      proposalId: data.proposalId,
      totalVotes: data.totalVotes,
      yesVotes: data.yesVotes,
      noVotes: data.noVotes,
      abstainVotes: data.abstainVotes,
      totalPower: data.totalPower,
      quorumAchieved: data.quorumAchieved,
      majorityAchieved: data.majorityAchieved,
      result: data.result,
      calculatedAt: new Date(data.calculatedAt),
    };
  }

  private parseExecutionResult(data: any): ExecutionResult {
    return {
      proposalId: data.proposalId,
      success: data.success,
      output: data.output,
      error: data.error,
      gasUsed: data.gasUsed,
      txHash: data.txHash,
      executedAt: new Date(data.executedAt),
      executor: data.executor,
    };
  }

  private parseDAOMember(data: any): DAOMember {
    return {
      address: data.address,
      did: data.did,
      reputation: data.reputation,
      stake: data.stake,
      votingPower: data.votingPower,
      status: data.status,
      joinedAt: new Date(data.joinedAt),
      lastActivity: new Date(data.lastActivity),
    };
  }

  private parseDAOStats(data: any): DAOStats {
    return {
      totalProposals: data.totalProposals,
      activeProposals: data.activeProposals,
      passedProposals: data.passedProposals,
      failedProposals: data.failedProposals,
      executedProposals: data.executedProposals,
      totalVotes: data.totalVotes,
      totalMembers: data.totalMembers,
      activeMembers: data.activeMembers,
      totalVotingPower: data.totalVotingPower,
      lastUpdated: new Date(data.lastUpdated),
    };
  }

  private parseGovernanceParams(data: any): GovernanceParams {
    return {
      minVotingPeriod: data.minVotingPeriod,
      maxVotingPeriod: data.maxVotingPeriod,
      minVotesRequired: data.minVotesRequired,
      majorityThreshold: data.majorityThreshold,
      quorumThreshold: data.quorumThreshold,
      executionDelay: data.executionDelay,
      proposalDeposit: data.proposalDeposit,
      votingPowerMethod: data.votingPowerMethod,
    };
  }
}

// Utility functions

/**
 * Create a new DAO instance
 */
export function createDAO(options?: {
  baseUrl?: string;
  apiKey?: string;
  timeout?: number;
}): AetherisDAO {
  return new AetherisDAO(options);
}

/**
 * Validate proposal data
 */
export function validateProposal(data: CreateProposalOptions): string[] {
  const errors: string[] = [];

  if (!data.title || data.title.trim().length === 0) {
    errors.push('Title is required');
  }

  if (!data.description || data.description.trim().length === 0) {
    errors.push('Description is required');
  }

  if (!data.proposer || data.proposer.trim().length === 0) {
    errors.push('Proposer address is required');
  }

  if (!data.proposerDID || data.proposerDID.trim().length === 0) {
    errors.push('Proposer DID is required');
  }

  if (data.votingPeriod && (data.votingPeriod < 3600 || data.votingPeriod > 2592000)) {
    errors.push('Voting period must be between 1 hour and 30 days');
  }

  return errors;
}

/**
 * Validate vote data
 */
export function validateVote(data: VoteOptions): string[] {
  const errors: string[] = [];

  if (!data.proposalId || data.proposalId.trim().length === 0) {
    errors.push('Proposal ID is required');
  }

  if (!data.voter || data.voter.trim().length === 0) {
    errors.push('Voter address is required');
  }

  if (!data.voterDID || data.voterDID.trim().length === 0) {
    errors.push('Voter DID is required');
  }

  if (!['yes', 'no', 'abstain'].includes(data.choice)) {
    errors.push('Vote choice must be yes, no, or abstain');
  }

  if (data.weight && data.weight <= 0) {
    errors.push('Vote weight must be greater than 0');
  }

  return errors;
}

/**
 * Format time remaining for a proposal
 */
export function formatTimeRemaining(endTime: Date): string {
  const now = new Date();
  const diff = endTime.getTime() - now.getTime();

  if (diff <= 0) {
    return 'Voting ended';
  }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

  if (days > 0) {
    return `${days}d ${hours}h ${minutes}m`;
  } else if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else {
    return `${minutes}m`;
  }
}

/**
 * Calculate proposal status
 */
export function calculateProposalStatus(proposal: Proposal): ProposalStatus {
  const now = new Date();
  
  if (proposal.status === 'draft') {
    return 'draft';
  }
  
  if (proposal.status === 'active' && now > proposal.endTime) {
    return 'voting-ended';
  }
  
  return proposal.status;
}

// Export default instance
export default AetherisDAO;

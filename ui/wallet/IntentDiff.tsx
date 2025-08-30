/**
 * Intent Diff Component
 * 
 * A React component that displays natural language explanations of transaction intents,
 * helping users understand "what they are about to sign" before confirming wallet operations.
 */

import React, { useState, useEffect, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  AlertTriangle,
  CheckCircle,
  XCircle,
  Info,
  Clock,
  DollarSign,
  Shield,
  ExternalLink,
  RefreshCw,
  Eye,
  EyeOff,
  Copy,
  Check
} from 'lucide-react';

// Type definitions matching the Rust backend
export interface TokenInfo {
  symbol: string;
  name: string;
  decimals: number;
  contractAddress?: string;
  chainId?: number;
  logoUrl?: string;
}

export interface TokenAmount {
  token: TokenInfo;
  amount: string;
  usdValue?: string;
}

export interface GasEstimate {
  gasLimit: number;
  gasPrice: string;
  totalFee: string;
  feeToken: TokenInfo;
  usdValue?: string;
}

export interface NetworkInfo {
  chainId: number;
  name: string;
  currency: string;
  blockExplorerUrl?: string;
}

export interface ContractParameter {
  name: string;
  paramType: string;
  value: any;
}

export type IntentOperation = 
  | {
      type: 'transfer';
      from: string;
      to: string;
      amount: string;
      token: TokenInfo;
      memo?: string;
    }
  | {
      type: 'swap';
      fromToken: TokenInfo;
      toToken: TokenInfo;
      fromAmount: string;
      minToAmount: string;
      slippageTolerance?: string;
      dex?: string;
    }
  | {
      type: 'contract_call';
      contractAddress: string;
      functionName: string;
      parameters: ContractParameter[];
      value?: string;
    }
  | {
      type: 'multi_sig';
      multisigAddress: string;
      operations: IntentOperation[];
      requiredSignatures: number;
      currentSignatures: number;
    }
  | {
      type: 'stake';
      validator: string;
      amount: string;
      token: TokenInfo;
      lockPeriod?: string;
    }
  | {
      type: 'unstake';
      validator: string;
      amount: string;
      token: TokenInfo;
      withdrawalDelay?: string;
    }
  | {
      type: 'defi_operation';
      protocol: string;
      operationType: string;
      tokens: TokenAmount[];
      parameters: Record<string, any>;
    };

export interface TransactionIntent {
  operations: IntentOperation[];
  gasEstimate?: GasEstimate;
  network: NetworkInfo;
  timestamp: string;
  nonce?: number;
  metadata: Record<string, any>;
}

export type RiskLevel = 'Low' | 'Medium' | 'High' | 'Critical';

export interface IntentSummary {
  primaryAction: string;
  details: string[];
  warnings: string[];
  networkInfo: string;
  feeSummary?: string;
  riskLevel: RiskLevel;
  confidence: number;
}

// Props for the IntentDiff component
export interface IntentDiffProps {
  /** The transaction intent to summarize */
  intent: TransactionIntent;
  /** Callback when user confirms the transaction */
  onConfirm: () => void;
  /** Callback when user rejects the transaction */
  onReject: () => void;
  /** Loading state */
  loading?: boolean;
  /** Show detailed view by default */
  showDetails?: boolean;
  /** Custom CSS classes */
  className?: string;
  /** Theme preference */
  theme?: 'light' | 'dark';
}

// Hook for fetching intent summary from Rust backend
function useIntentSummary(intent: TransactionIntent) {
  const [summary, setSummary] = useState<IntentSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isCancelled = false;

    const fetchSummary = async () => {
      try {
        setLoading(true);
        setError(null);

        // Call Rust backend via WebAssembly or HTTP API
        const response = await fetch('/api/wallet/summarize-intent', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(intent),
        });

        if (!response.ok) {
          throw new Error(`Failed to summarize intent: ${response.statusText}`);
        }

        const summaryData: IntentSummary = await response.json();
        
        if (!isCancelled) {
          setSummary(summaryData);
        }
      } catch (err) {
        if (!isCancelled) {
          setError(err instanceof Error ? err.message : 'Unknown error');
          console.error('Intent summarization error:', err);
        }
      } finally {
        if (!isCancelled) {
          setLoading(false);
        }
      }
    };

    fetchSummary();

    return () => {
      isCancelled = true;
    };
  }, [intent]);

  return { summary, loading, error };
}

// Risk level styling
const getRiskLevelStyles = (riskLevel: RiskLevel, theme: 'light' | 'dark' = 'light') => {
  const baseStyles = 'px-3 py-1 rounded-full text-sm font-medium flex items-center gap-1';
  
  switch (riskLevel) {
    case 'Low':
      return `${baseStyles} ${theme === 'light' 
        ? 'bg-green-100 text-green-800 border border-green-200' 
        : 'bg-green-900/30 text-green-300 border border-green-700'}`;
    case 'Medium':
      return `${baseStyles} ${theme === 'light'
        ? 'bg-yellow-100 text-yellow-800 border border-yellow-200'
        : 'bg-yellow-900/30 text-yellow-300 border border-yellow-700'}`;
    case 'High':
      return `${baseStyles} ${theme === 'light'
        ? 'bg-orange-100 text-orange-800 border border-orange-200'
        : 'bg-orange-900/30 text-orange-300 border border-orange-700'}`;
    case 'Critical':
      return `${baseStyles} ${theme === 'light'
        ? 'bg-red-100 text-red-800 border border-red-200'
        : 'bg-red-900/30 text-red-300 border border-red-700'}`;
    default:
      return `${baseStyles} ${theme === 'light'
        ? 'bg-gray-100 text-gray-800 border border-gray-200'
        : 'bg-gray-900/30 text-gray-300 border border-gray-700'}`;
  }
};

// Risk level icon
const getRiskLevelIcon = (riskLevel: RiskLevel) => {
  switch (riskLevel) {
    case 'Low':
      return <CheckCircle className="w-4 h-4" />;
    case 'Medium':
      return <Info className="w-4 h-4" />;
    case 'High':
    case 'Critical':
      return <AlertTriangle className="w-4 h-4" />;
    default:
      return <Info className="w-4 h-4" />;
  }
};

// Address formatting utility
const formatAddress = (address: string): string => {
  if (address.length <= 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

// Copy to clipboard hook
function useCopyToClipboard() {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return { copied, copyToClipboard };
}

// Loading skeleton component
const LoadingSkeleton: React.FC<{ theme?: 'light' | 'dark' }> = ({ theme = 'light' }) => (
  <div className="space-y-4 animate-pulse">
    <div className={`h-8 rounded-lg ${theme === 'light' ? 'bg-gray-200' : 'bg-gray-700'}`} />
    <div className="space-y-2">
      <div className={`h-4 rounded ${theme === 'light' ? 'bg-gray-200' : 'bg-gray-700'}`} />
      <div className={`h-4 rounded w-3/4 ${theme === 'light' ? 'bg-gray-200' : 'bg-gray-700'}`} />
    </div>
    <div className={`h-20 rounded-lg ${theme === 'light' ? 'bg-gray-200' : 'bg-gray-700'}`} />
  </div>
);

// Error component
const ErrorState: React.FC<{ error: string; onRetry: () => void; theme?: 'light' | 'dark' }> = ({ 
  error, 
  onRetry, 
  theme = 'light' 
}) => (
  <div className={`p-6 rounded-lg border-2 border-dashed text-center ${
    theme === 'light' 
      ? 'border-red-300 bg-red-50 text-red-800' 
      : 'border-red-700 bg-red-900/20 text-red-300'
  }`}>
    <XCircle className="w-12 h-12 mx-auto mb-4 text-red-500" />
    <h3 className="text-lg font-semibold mb-2">Failed to summarize transaction</h3>
    <p className="text-sm mb-4">{error}</p>
    <button
      onClick={onRetry}
      className={`inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${
        theme === 'light'
          ? 'bg-red-600 text-white hover:bg-red-700'
          : 'bg-red-600 text-white hover:bg-red-500'
      }`}
    >
      <RefreshCw className="w-4 h-4" />
      Retry
    </button>
  </div>
);

// Main IntentDiff component
export const IntentDiff: React.FC<IntentDiffProps> = ({
  intent,
  onConfirm,
  onReject,
  loading: externalLoading = false,
  showDetails: defaultShowDetails = false,
  className = '',
  theme = 'light'
}) => {
  const { summary, loading: summaryLoading, error } = useIntentSummary(intent);
  const [showDetails, setShowDetails] = useState(defaultShowDetails);
  const [showRawData, setShowRawData] = useState(false);
  const { copied, copyToClipboard } = useCopyToClipboard();
  
  const loading = externalLoading || summaryLoading;

  // Theme-based styling
  const themeStyles = useMemo(() => ({
    container: theme === 'light' 
      ? 'bg-white border border-gray-200 text-gray-900' 
      : 'bg-gray-900 border border-gray-700 text-gray-100',
    card: theme === 'light'
      ? 'bg-gray-50 border border-gray-200'
      : 'bg-gray-800 border border-gray-600',
    button: {
      primary: theme === 'light'
        ? 'bg-blue-600 text-white hover:bg-blue-700'
        : 'bg-blue-600 text-white hover:bg-blue-500',
      secondary: theme === 'light'
        ? 'bg-gray-200 text-gray-800 hover:bg-gray-300'
        : 'bg-gray-700 text-gray-200 hover:bg-gray-600',
      danger: theme === 'light'
        ? 'bg-red-600 text-white hover:bg-red-700'
        : 'bg-red-600 text-white hover:bg-red-500'
    }
  }), [theme]);

  // Retry function for error state
  const handleRetry = () => {
    window.location.reload(); // Simple retry - in production, you'd re-fetch
  };

  if (loading) {
    return (
      <div className={`p-6 rounded-xl shadow-lg ${themeStyles.container} ${className}`}>
        <LoadingSkeleton theme={theme} />
      </div>
    );
  }

  if (error) {
    return (
      <div className={`p-6 rounded-xl shadow-lg ${themeStyles.container} ${className}`}>
        <ErrorState error={error} onRetry={handleRetry} theme={theme} />
      </div>
    );
  }

  if (!summary) {
    return (
      <div className={`p-6 rounded-xl shadow-lg ${themeStyles.container} ${className}`}>
        <div className="text-center py-8">
          <Info className="w-12 h-12 mx-auto mb-4 text-gray-400" />
          <p>No summary available</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`rounded-xl shadow-lg ${themeStyles.container} ${className}`}>
      {/* Header */}
      <div className="p-6 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1">
            <h2 className="text-xl font-bold mb-2">Transaction Summary</h2>
            <p className="text-lg">{summary.primaryAction}</p>
          </div>
          <div className="flex items-center gap-2">
            <div className={getRiskLevelStyles(summary.riskLevel, theme)}>
              {getRiskLevelIcon(summary.riskLevel)}
              {summary.riskLevel} Risk
            </div>
            <div className={`px-3 py-1 rounded-full text-sm ${
              theme === 'light' ? 'bg-blue-100 text-blue-800' : 'bg-blue-900/30 text-blue-300'
            }`}>
              {Math.round(summary.confidence * 100)}% Confidence
            </div>
          </div>
        </div>

        {/* Network info */}
        <div className="mt-4 flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
          <div className="flex items-center gap-1">
            <Shield className="w-4 h-4" />
            {summary.networkInfo}
          </div>
          {summary.feeSummary && (
            <div className="flex items-center gap-1">
              <DollarSign className="w-4 h-4" />
              {summary.feeSummary}
            </div>
          )}
          <div className="flex items-center gap-1">
            <Clock className="w-4 h-4" />
            {new Date(intent.timestamp).toLocaleString()}
          </div>
        </div>
      </div>

      {/* Warnings */}
      <AnimatePresence>
        {summary.warnings.length > 0 && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className={`p-4 border-b border-gray-200 dark:border-gray-700 ${
              theme === 'light' ? 'bg-amber-50' : 'bg-amber-900/10'
            }`}
          >
            <div className="flex items-start gap-3">
              <AlertTriangle className="w-5 h-5 text-amber-500 mt-0.5 flex-shrink-0" />
              <div className="flex-1">
                <h3 className="font-semibold text-amber-800 dark:text-amber-300 mb-2">
                  Important Warnings
                </h3>
                <ul className="space-y-1 text-sm text-amber-700 dark:text-amber-400">
                  {summary.warnings.map((warning, index) => (
                    <li key={index} className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-amber-500 rounded-full mt-2 flex-shrink-0" />
                      {warning}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Details */}
      <div className="p-6">
        <button
          onClick={() => setShowDetails(!showDetails)}
          className="flex items-center gap-2 text-sm font-medium text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 mb-4"
        >
          {showDetails ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          {showDetails ? 'Hide Details' : 'Show Details'}
        </button>

        <AnimatePresence>
          {showDetails && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="space-y-4"
            >
              <div className={`p-4 rounded-lg ${themeStyles.card}`}>
                <h4 className="font-semibold mb-3">Operation Details</h4>
                <ul className="space-y-2">
                  {summary.details.map((detail, index) => (
                    <li key={index} className="flex items-start gap-2 text-sm">
                      <span className="w-1 h-1 bg-gray-400 rounded-full mt-2 flex-shrink-0" />
                      {detail}
                    </li>
                  ))}
                </ul>
              </div>

              {/* Raw transaction data toggle */}
              <div>
                <button
                  onClick={() => setShowRawData(!showRawData)}
                  className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
                >
                  {showRawData ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
                  {showRawData ? 'Hide' : 'Show'} Raw Transaction Data
                </button>

                <AnimatePresence>
                  {showRawData && (
                    <motion.div
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: 'auto' }}
                      exit={{ opacity: 0, height: 0 }}
                      className="mt-2"
                    >
                      <div className={`p-3 rounded border text-xs font-mono relative ${
                        theme === 'light' ? 'bg-gray-100 border-gray-300' : 'bg-gray-800 border-gray-600'
                      }`}>
                        <button
                          onClick={() => copyToClipboard(JSON.stringify(intent, null, 2))}
                          className="absolute top-2 right-2 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                        >
                          {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
                        </button>
                        <pre className="whitespace-pre-wrap break-all overflow-x-auto pr-8">
                          {JSON.stringify(intent, null, 2)}
                        </pre>
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Action buttons */}
      <div className="p-6 border-t border-gray-200 dark:border-gray-700 flex gap-3">
        <button
          onClick={onReject}
          className={`px-6 py-3 rounded-lg font-semibold transition-colors flex-1 ${themeStyles.button.secondary}`}
        >
          Reject Transaction
        </button>
        <button
          onClick={onConfirm}
          className={`px-6 py-3 rounded-lg font-semibold transition-colors flex-1 ${
            summary.riskLevel === 'Critical' 
              ? themeStyles.button.danger 
              : themeStyles.button.primary
          }`}
        >
          {summary.riskLevel === 'Critical' ? 'Sign Anyway' : 'Confirm & Sign'}
        </button>
      </div>
    </div>
  );
};

// Export utility functions for testing
export { formatAddress, getRiskLevelStyles, getRiskLevelIcon };

// Export hook for external use
export { useIntentSummary };

export default IntentDiff;

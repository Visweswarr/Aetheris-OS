import React, { useState } from 'react';
import { ConsentCard, ConsentRequest, ConsentDecision } from './ConsentCard';

// Sample consent requests for demonstration
const sampleConsentRequests: ConsentRequest[] = [
  {
    token: {
      header: {
        typ: "capability",
        alg: "Dilithium3",
        ver: "1.0.0",
        kid: "key-123",
        jti: "token-456",
        iss: "service.example.com",
        sub: "user-789",
        aud: "app.example.com",
        iat: Math.floor(Date.now() / 1000),
        nbf: Math.floor(Date.now() / 1000),
        exp: Math.floor(Date.now() / 1000) + 3600,
        additional: {},
      },
      payload: {
        purpose: "Access user profile and preferences",
        claims: [
          {
            claim_type: "Resource",
            claim_data: {
              resource_type: "file",
              resource_id: "user-profile.json",
              resource_path: "/users/789/profile.json",
              permissions: ["read", "write"],
            },
            constraints: {},
            metadata: {},
          },
          {
            claim_type: "Action",
            claim_data: {
              action_name: "update_profile",
              parameters: { fields: ["name", "email"] },
              constraints: {},
              metadata: {},
            },
            constraints: {},
            metadata: {},
          },
        ],
        scope: "user_profile",
        level: 2,
        hierarchy: ["user", "profile"],
        constraints: {},
        metadata: {},
        additional: {},
      },
      signature: null,
      format_version: "1.0.0",
    },
    request_id: "req-123",
    requested_at: Math.floor(Date.now() / 1000),
    expires_at: Math.floor(Date.now() / 1000) + 3600,
    requester: {
      name: "Example App",
      id: "app-123",
      domain: "app.example.com",
      logo: "https://via.placeholder.com/40x40/3B82F6/FFFFFF?text=E",
    },
    context: {
      application: "User Profile Manager",
      action: "Update user profile information",
      description: "This app needs access to read and update your profile information to provide personalized services.",
      urgency: "medium",
    },
  },
  {
    token: {
      header: {
        typ: "capability",
        alg: "Dilithium3",
        ver: "1.0.0",
        kid: "key-456",
        jti: "token-789",
        iss: "service.example.com",
        sub: "user-789",
        aud: "analytics.example.com",
        iat: Math.floor(Date.now() / 1000),
        nbf: Math.floor(Date.now() / 1000),
        exp: Math.floor(Date.now() / 1000) + 7200,
        additional: {},
      },
      payload: {
        purpose: "Analytics and performance monitoring",
        claims: [
          {
            claim_type: "Data",
            claim_data: {
              data_type: "analytics",
              data_classification: "public",
              access_level: 1,
              data_constraints: {},
            },
            constraints: {},
            metadata: {},
          },
          {
            claim_type: "Location",
            claim_data: {
              coordinates: [40.7128, -74.0060],
              region: "US",
              network_location: null,
              location_constraints: {},
            },
            constraints: {},
            metadata: {},
          },
          {
            claim_type: "Device",
            claim_data: {
              device_type: "web",
              device_id: "browser-123",
              device_capabilities: ["javascript", "cookies"],
              device_constraints: {},
            },
            constraints: {},
            metadata: {},
          },
        ],
        scope: "analytics",
        level: 1,
        hierarchy: ["analytics"],
        constraints: {},
        metadata: {},
        additional: {},
      },
      signature: null,
      format_version: "1.0.0",
    },
    request_id: "req-456",
    requested_at: Math.floor(Date.now() / 1000),
    expires_at: Math.floor(Date.now() / 1000) + 7200,
    requester: {
      name: "Analytics Service",
      id: "analytics-123",
      domain: "analytics.example.com",
      logo: "https://via.placeholder.com/40x40/10B981/FFFFFF?text=A",
    },
    context: {
      application: "Performance Analytics",
      action: "Collect usage analytics",
      description: "We collect anonymous usage data to improve our services and monitor performance.",
      urgency: "low",
    },
  },
  {
    token: {
      header: {
        typ: "capability",
        alg: "Dilithium3",
        ver: "1.0.0",
        kid: "key-789",
        jti: "token-012",
        iss: "service.example.com",
        sub: "user-789",
        aud: "payment.example.com",
        iat: Math.floor(Date.now() / 1000),
        nbf: Math.floor(Date.now() / 1000),
        exp: Math.floor(Date.now() / 1000) + 1800,
        additional: {},
      },
      payload: {
        purpose: "Process payment transaction",
        claims: [
          {
            claim_type: "Resource",
            claim_data: {
              resource_type: "payment_method",
              resource_id: "card-123",
              resource_path: "/users/789/payment-methods/card-123",
              permissions: ["read"],
            },
            constraints: {},
            metadata: {},
          },
          {
            claim_type: "Action",
            claim_data: {
              action_name: "process_payment",
              parameters: { amount: 29.99, currency: "USD" },
              constraints: {},
              metadata: {},
            },
            constraints: {},
            metadata: {},
          },
          {
            claim_type: "Data",
            claim_data: {
              data_type: "payment",
              data_classification: "sensitive",
              access_level: 3,
              data_constraints: {},
            },
            constraints: {},
            metadata: {},
          },
        ],
        scope: "payment",
        level: 3,
        hierarchy: ["payment"],
        constraints: {},
        metadata: {},
        additional: {},
      },
      signature: null,
      format_version: "1.0.0",
    },
    request_id: "req-789",
    requested_at: Math.floor(Date.now() / 1000),
    expires_at: Math.floor(Date.now() / 1000) + 1800,
    requester: {
      name: "Payment Processor",
      id: "payment-123",
      domain: "payment.example.com",
      logo: "https://via.placeholder.com/40x40/EF4444/FFFFFF?text=P",
    },
    context: {
      application: "Payment Gateway",
      action: "Process subscription payment",
      description: "We need access to your payment method to process your monthly subscription payment.",
      urgency: "high",
    },
  },
];

export const ConsentCardDemo: React.FC = () => {
  const [selectedRequest, setSelectedRequest] = useState<ConsentRequest>(sampleConsentRequests[0]);
  const [decisions, setDecisions] = useState<Record<string, ConsentDecision>>({});
  const [showResults, setShowResults] = useState(false);

  const handleDecision = (decision: ConsentDecision) => {
    setDecisions(prev => ({
      ...prev,
      [decision.request_id]: decision,
    }));
    setShowResults(true);
  };

  const handleCancel = () => {
    setShowResults(false);
  };

  const getDecisionSummary = (decision: ConsentDecision) => {
    switch (decision.decision) {
      case 'granted':
        return `✅ Granted all ${decision.granted_claims.length} capabilities`;
      case 'denied':
        return `❌ Denied all ${decision.denied_claims.length} capabilities`;
      case 'partial':
        return `⚠️ Partially granted: ${decision.granted_claims.length} granted, ${decision.denied_claims.length} denied`;
      default:
        return 'Unknown decision';
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-6xl mx-auto px-4">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-4">
            Consent UX Demo
          </h1>
          <p className="text-lg text-gray-600 max-w-2xl mx-auto">
            Experience the human-readable capability consent interface. Select different consent requests 
            to see how the system renders various capability types and handles user decisions.
          </p>
        </div>

        {/* Request Selector */}
        <div className="bg-white rounded-lg shadow-sm border p-6 mb-8">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Select Consent Request
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {sampleConsentRequests.map((request, index) => (
              <button
                key={request.request_id}
                onClick={() => setSelectedRequest(request)}
                className={`p-4 rounded-lg border-2 text-left transition-colors ${
                  selectedRequest.request_id === request.request_id
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300'
                }`}
              >
                <div className="flex items-center space-x-3 mb-2">
                  {request.requester.logo && (
                    <img
                      src={request.requester.logo}
                      alt={request.requester.name}
                      className="w-8 h-8 rounded-full"
                    />
                  )}
                  <div>
                    <h3 className="font-medium text-gray-900">
                      {request.requester.name}
                    </h3>
                    <p className="text-sm text-gray-500">
                      {request.requester.domain}
                    </p>
                  </div>
                </div>
                <p className="text-sm text-gray-600 mb-2">
                  {request.context.description}
                </p>
                <div className="flex items-center justify-between">
                  <span className="text-xs text-gray-500">
                    {request.token.payload.claims.length} capabilities
                  </span>
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                    request.context.urgency === 'low' ? 'bg-green-100 text-green-800' :
                    request.context.urgency === 'medium' ? 'bg-yellow-100 text-yellow-800' :
                    request.context.urgency === 'high' ? 'bg-orange-100 text-orange-800' :
                    'bg-red-100 text-red-800'
                  }`}>
                    {request.context.urgency}
                  </span>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Consent Card */}
        <div className="mb-8">
          <ConsentCard
            consentRequest={selectedRequest}
            onDecision={handleDecision}
            onCancel={handleCancel}
            className="shadow-lg"
          />
        </div>

        {/* Results */}
        {showResults && Object.keys(decisions).length > 0 && (
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h2 className="text-xl font-semibold text-gray-900 mb-4">
              Consent Decisions
            </h2>
            <div className="space-y-4">
              {Object.entries(decisions).map(([requestId, decision]) => {
                const request = sampleConsentRequests.find(r => r.request_id === requestId);
                return (
                  <div key={requestId} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="font-medium text-gray-900">
                        {request?.requester.name || 'Unknown'}
                      </h3>
                      <span className="text-sm text-gray-500">
                        {new Date(decision.timestamp * 1000).toLocaleString()}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 mb-2">
                      {getDecisionSummary(decision)}
                    </p>
                    {decision.notes && (
                      <p className="text-sm text-gray-600 mb-2">
                        <strong>Notes:</strong> {decision.notes}
                      </p>
                    )}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                      <div>
                        <strong className="text-gray-700">Granted:</strong>
                        <ul className="list-disc list-inside text-gray-600 mt-1">
                          {decision.granted_claims.map(claim => (
                            <li key={claim}>{claim}</li>
                          ))}
                        </ul>
                      </div>
                      <div>
                        <strong className="text-gray-700">Denied:</strong>
                        <ul className="list-disc list-inside text-gray-600 mt-1">
                          {decision.denied_claims.map(claim => (
                            <li key={claim}>{claim}</li>
                          ))}
                        </ul>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
            <div className="mt-6 pt-4 border-t">
              <button
                onClick={() => setDecisions({})}
                className="px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors"
              >
                Clear All Decisions
              </button>
            </div>
          </div>
        )}

        {/* Feature Overview */}
        <div className="bg-white rounded-lg shadow-sm border p-6 mt-8">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Feature Overview
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-medium text-gray-900 mb-2">Human-Readable Summaries</h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Clear purpose and context descriptions</li>
                <li>• Plain language capability explanations</li>
                <li>• Visual icons for different capability types</li>
                <li>• Urgency indicators and expiration information</li>
              </ul>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 mb-2">Granular Consent Control</h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Individual capability toggles</li>
                <li>• Bulk grant/deny actions</li>
                <li>• Partial consent support</li>
                <li>• Custom expiration settings</li>
              </ul>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 mb-2">Advanced Options</h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Detailed capability information</li>
                <li>• Constraint and metadata display</li>
                <li>• Notes and custom parameters</li>
                <li>• Responsive design for all devices</li>
              </ul>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 mb-2">Security Features</h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Post-quantum signature support</li>
                <li>• Expiration validation</li>
                <li>• Tampering detection</li>
                <li>• Audit trail and logging</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ConsentCardDemo;

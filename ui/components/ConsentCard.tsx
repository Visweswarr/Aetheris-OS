import React, { useState, useCallback, useMemo } from 'react';
import { 
  Card, 
  CardContent, 
  CardHeader, 
  CardTitle, 
  CardDescription,
  Button, 
  ButtonGroup,
  Badge,
  Separator,
  Alert,
  AlertDescription,
  Checkbox,
  Label,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './ui';
import { 
  Shield, 
  Clock, 
  MapPin, 
  Smartphone, 
  Wifi, 
  Database, 
  FileText, 
  Settings,
  Info,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Lock,
  User,
  Calendar,
  Globe,
  HardDrive,
  Network,
  Smartphone as DeviceIcon,
  Eye,
  EyeOff,
} from 'lucide-react';

// Types for capability tokens
export interface CapabilityClaim {
  claim_type: 'Resource' | 'Action' | 'Scope' | 'Time' | 'Location' | 'Device' | 'Network' | 'Data' | 'Custom';
  claim_data: any;
  constraints: Record<string, any>;
  metadata: Record<string, any>;
}

export interface CapabilityToken {
  header: {
    typ: string;
    alg: string;
    ver: string;
    kid: string;
    jti: string;
    iss: string;
    sub: string;
    aud: string;
    iat: number;
    nbf: number;
    exp: number;
    additional: Record<string, any>;
  };
  payload: {
    purpose: string;
    claims: CapabilityClaim[];
    scope: string;
    level: number;
    hierarchy: string[];
    constraints: Record<string, any>;
    metadata: Record<string, any>;
    additional: Record<string, any>;
  };
  signature?: any;
  format_version: string;
}

export interface ConsentRequest {
  token: CapabilityToken;
  request_id: string;
  requested_at: number;
  expires_at: number;
  requester: {
    name: string;
    id: string;
    domain: string;
    logo?: string;
  };
  context: {
    application: string;
    action: string;
    description: string;
    urgency: 'low' | 'medium' | 'high' | 'critical';
  };
}

export interface ConsentDecision {
  request_id: string;
  decision: 'granted' | 'denied' | 'partial';
  granted_claims: string[];
  denied_claims: string[];
  expires_at?: number;
  notes?: string;
  timestamp: number;
}

// Props for the ConsentCard component
export interface ConsentCardProps {
  consentRequest: ConsentRequest;
  onDecision: (decision: ConsentDecision) => void;
  onCancel?: () => void;
  showAdvanced?: boolean;
  className?: string;
  disabled?: boolean;
}

// Utility functions for human-readable descriptions
const formatTimestamp = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
};

const formatDuration = (seconds: number): string => {
  if (seconds < 60) return `${seconds} seconds`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)} minutes`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} hours`;
  return `${Math.floor(seconds / 86400)} days`;
};

const getUrgencyColor = (urgency: string): string => {
  switch (urgency) {
    case 'low': return 'bg-green-100 text-green-800';
    case 'medium': return 'bg-yellow-100 text-yellow-800';
    case 'high': return 'bg-orange-100 text-orange-800';
    case 'critical': return 'bg-red-100 text-red-800';
    default: return 'bg-gray-100 text-gray-800';
  }
};

const getUrgencyIcon = (urgency: string) => {
  switch (urgency) {
    case 'low': return <CheckCircle className="w-4 h-4" />;
    case 'medium': return <AlertTriangle className="w-4 h-4" />;
    case 'high': return <AlertTriangle className="w-4 h-4" />;
    case 'critical': return <XCircle className="w-4 h-4" />;
    default: return <Info className="w-4 h-4" />;
  }
};

// Component for rendering individual capability claims
const CapabilityClaimItem: React.FC<{
  claim: CapabilityClaim;
  index: number;
  isGranted: boolean;
  onToggle: (index: number, granted: boolean) => void;
}> = ({ claim, index, isGranted, onToggle }) => {
  const [showDetails, setShowDetails] = useState(false);

  const getClaimIcon = (claimType: string) => {
    switch (claimType) {
      case 'Resource': return <FileText className="w-4 h-4" />;
      case 'Action': return <Settings className="w-4 h-4" />;
      case 'Scope': return <User className="w-4 h-4" />;
      case 'Time': return <Clock className="w-4 h-4" />;
      case 'Location': return <MapPin className="w-4 h-4" />;
      case 'Device': return <DeviceIcon className="w-4 h-4" />;
      case 'Network': return <Network className="w-4 h-4" />;
      case 'Data': return <Database className="w-4 h-4" />;
      case 'Custom': return <Settings className="w-4 h-4" />;
      default: return <Info className="w-4 h-4" />;
    }
  };

  const getClaimDescription = (claim: CapabilityClaim): string => {
    switch (claim.claim_type) {
      case 'Resource':
        const resource = claim.claim_data;
        return `Access to ${resource.resource_type} "${resource.resource_id}" at ${resource.resource_path} with permissions: ${resource.permissions.join(', ')}`;
      
      case 'Action':
        const action = claim.claim_data;
        return `Perform action "${action.action_name}" with parameters: ${Object.keys(action.parameters).length > 0 ? Object.keys(action.parameters).join(', ') : 'none'}`;
      
      case 'Scope':
        const scope = claim.claim_data;
        return `Access scope "${scope.scope_name}" at level ${scope.scope_level} in hierarchy: ${scope.scope_hierarchy.join(' > ')}`;
      
      case 'Time':
        const time = claim.claim_data;
        const from = formatTimestamp(time.valid_from);
        const until = formatTimestamp(time.valid_until);
        return `Valid from ${from} until ${until}`;
      
      case 'Location':
        const location = claim.claim_data;
        if (location.coordinates) {
          const [lat, lng] = location.coordinates;
          return `Access from location: ${lat.toFixed(4)}, ${lng.toFixed(4)}${location.region ? ` in ${location.region}` : ''}`;
        }
        return `Access from region: ${location.region || 'any'}`;
      
      case 'Device':
        const device = claim.claim_data;
        return `Access from device "${device.device_id}" (${device.device_type}) with capabilities: ${device.device_capabilities.join(', ')}`;
      
      case 'Network':
        const network = claim.claim_data;
        return `Access from network "${network.network_id}" (${network.network_type}) at security level ${network.security_level}`;
      
      case 'Data':
        const data = claim.claim_data;
        return `Access to ${data.data_type} data classified as "${data.data_classification}" at access level ${data.access_level}`;
      
      case 'Custom':
        const custom = claim.claim_data;
        return `Custom capability "${custom.claim_name}": ${JSON.stringify(custom.claim_value)}`;
      
      default:
        return `Unknown capability type: ${claim.claim_type}`;
    }
  };

  const getClaimSummary = (claim: CapabilityClaim): string => {
    switch (claim.claim_type) {
      case 'Resource':
        const resource = claim.claim_data;
        return `${resource.resource_type}: ${resource.resource_id}`;
      
      case 'Action':
        const action = claim.claim_data;
        return `Action: ${action.action_name}`;
      
      case 'Scope':
        const scope = claim.claim_data;
        return `Scope: ${scope.scope_name} (Level ${scope.scope_level})`;
      
      case 'Time':
        const time = claim.claim_data;
        const duration = time.valid_until - time.valid_from;
        return `Time: ${formatDuration(duration)}`;
      
      case 'Location':
        const location = claim.claim_data;
        return `Location: ${location.region || 'Any'}`;
      
      case 'Device':
        const device = claim.claim_data;
        return `Device: ${device.device_type}`;
      
      case 'Network':
        const network = claim.claim_data;
        return `Network: ${network.network_type}`;
      
      case 'Data':
        const data = claim.claim_data;
        return `Data: ${data.data_type} (${data.data_classification})`;
      
      case 'Custom':
        const custom = claim.claim_data;
        return `Custom: ${custom.claim_name}`;
      
      default:
        return `Unknown: ${claim.claim_type}`;
    }
  };

  return (
    <div className="border rounded-lg p-4 mb-3 bg-gray-50">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2">
          {getClaimIcon(claim.claim_type)}
          <span className="font-medium text-sm text-gray-700">
            {getClaimSummary(claim)}
          </span>
          <Badge variant={isGranted ? "default" : "secondary"}>
            {isGranted ? "Granted" : "Denied"}
          </Badge>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox
            id={`claim-${index}`}
            checked={isGranted}
            onCheckedChange={(checked) => onToggle(index, checked as boolean)}
          />
          <Label htmlFor={`claim-${index}`} className="text-sm">
            {isGranted ? "Grant" : "Deny"}
          </Label>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowDetails(!showDetails)}
          >
            {showDetails ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          </Button>
        </div>
      </div>
      
      {showDetails && (
        <div className="mt-3 p-3 bg-white rounded border">
          <p className="text-sm text-gray-600 mb-2">
            {getClaimDescription(claim)}
          </p>
          
          {Object.keys(claim.constraints).length > 0 && (
            <div className="mt-2">
              <p className="text-xs font-medium text-gray-500 mb-1">Constraints:</p>
              <div className="text-xs text-gray-600">
                {Object.entries(claim.constraints).map(([key, value]) => (
                  <div key={key} className="flex justify-between">
                    <span>{key}:</span>
                    <span>{JSON.stringify(value)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
          
          {Object.keys(claim.metadata).length > 0 && (
            <div className="mt-2">
              <p className="text-xs font-medium text-gray-500 mb-1">Metadata:</p>
              <div className="text-xs text-gray-600">
                {Object.entries(claim.metadata).map(([key, value]) => (
                  <div key={key} className="flex justify-between">
                    <span>{key}:</span>
                    <span>{JSON.stringify(value)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

// Main ConsentCard component
export const ConsentCard: React.FC<ConsentCardProps> = ({
  consentRequest,
  onDecision,
  onCancel,
  showAdvanced = false,
  className = '',
  disabled = false,
}) => {
  const [grantedClaims, setGrantedClaims] = useState<Set<number>>(new Set());
  const [showAdvancedDetails, setShowAdvancedDetails] = useState(showAdvanced);
  const [notes, setNotes] = useState('');
  const [customExpiration, setCustomExpiration] = useState<number | undefined>();

  // Initialize all claims as granted by default
  React.useEffect(() => {
    const allGranted = new Set(consentRequest.token.payload.claims.map((_, index) => index));
    setGrantedClaims(allGranted);
  }, [consentRequest]);

  const handleClaimToggle = useCallback((index: number, granted: boolean) => {
    setGrantedClaims(prev => {
      const newSet = new Set(prev);
      if (granted) {
        newSet.add(index);
      } else {
        newSet.delete(index);
      }
      return newSet;
    });
  }, []);

  const handleGrantAll = useCallback(() => {
    const allGranted = new Set(consentRequest.token.payload.claims.map((_, index) => index));
    setGrantedClaims(allGranted);
  }, [consentRequest]);

  const handleDenyAll = useCallback(() => {
    setGrantedClaims(new Set());
  }, []);

  const handleGrant = useCallback(() => {
    const decision: ConsentDecision = {
      request_id: consentRequest.request_id,
      decision: grantedClaims.size === consentRequest.token.payload.claims.length ? 'granted' : 'partial',
      granted_claims: Array.from(grantedClaims).map(index => 
        consentRequest.token.payload.claims[index].claim_type
      ),
      denied_claims: consentRequest.token.payload.claims
        .map((_, index) => index)
        .filter(index => !grantedClaims.has(index))
        .map(index => consentRequest.token.payload.claims[index].claim_type),
      expires_at: customExpiration || consentRequest.expires_at,
      notes: notes.trim() || undefined,
      timestamp: Math.floor(Date.now() / 1000),
    };
    onDecision(decision);
  }, [consentRequest, grantedClaims, customExpiration, notes, onDecision]);

  const handleDeny = useCallback(() => {
    const decision: ConsentDecision = {
      request_id: consentRequest.request_id,
      decision: 'denied',
      granted_claims: [],
      denied_claims: consentRequest.token.payload.claims.map(claim => claim.claim_type),
      notes: notes.trim() || undefined,
      timestamp: Math.floor(Date.now() / 1000),
    };
    onDecision(decision);
  }, [consentRequest, notes, onDecision]);

  const isPartiallyGranted = grantedClaims.size > 0 && grantedClaims.size < consentRequest.token.payload.claims.length;
  const canGrant = grantedClaims.size > 0;

  const expirationDate = useMemo(() => {
    const timestamp = customExpiration || consentRequest.expires_at;
    return new Date(timestamp * 1000);
  }, [customExpiration, consentRequest.expires_at]);

  const isExpired = useMemo(() => {
    return Date.now() > expirationDate.getTime();
  }, [expirationDate]);

  if (isExpired) {
    return (
      <Card className={`${className} border-red-200 bg-red-50`}>
        <CardContent className="p-6">
          <div className="text-center">
            <XCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-red-800 mb-2">
              Consent Request Expired
            </h3>
            <p className="text-red-600 mb-4">
              This consent request expired on {expirationDate.toLocaleString()}
            </p>
            <Button variant="outline" onClick={onCancel}>
              Close
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <TooltipProvider>
      <Card className={`${className} max-w-2xl mx-auto`}>
        <CardHeader className="pb-4">
          <div className="flex items-start justify-between">
            <div className="flex items-center space-x-3">
              {consentRequest.requester.logo ? (
                <img 
                  src={consentRequest.requester.logo} 
                  alt={consentRequest.requester.name}
                  className="w-10 h-10 rounded-full"
                />
              ) : (
                <div className="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
                  <Shield className="w-5 h-5 text-blue-600" />
                </div>
              )}
              <div>
                <CardTitle className="text-xl">
                  {consentRequest.requester.name}
                </CardTitle>
                <CardDescription className="text-sm">
                  {consentRequest.requester.domain}
                </CardDescription>
              </div>
            </div>
            <Badge className={getUrgencyColor(consentRequest.context.urgency)}>
              <div className="flex items-center space-x-1">
                {getUrgencyIcon(consentRequest.context.urgency)}
                <span className="capitalize">{consentRequest.context.urgency}</span>
              </div>
            </Badge>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Request Context */}
          <div className="bg-blue-50 p-4 rounded-lg border border-blue-200">
            <h4 className="font-medium text-blue-900 mb-2">
              {consentRequest.context.application}
            </h4>
            <p className="text-blue-800 text-sm mb-2">
              {consentRequest.context.description}
            </p>
            <p className="text-blue-700 text-sm">
              <strong>Action:</strong> {consentRequest.context.action}
            </p>
          </div>

          {/* Token Purpose */}
          <div className="flex items-center space-x-2">
            <Lock className="w-5 h-5 text-gray-500" />
            <div>
              <p className="font-medium text-gray-900">Purpose</p>
              <p className="text-sm text-gray-600">{consentRequest.token.payload.purpose}</p>
            </div>
          </div>

          {/* Token Scope */}
          <div className="flex items-center space-x-2">
            <User className="w-5 h-5 text-gray-500" />
            <div>
              <p className="font-medium text-gray-900">Scope</p>
              <p className="text-sm text-gray-600">{consentRequest.token.payload.scope}</p>
            </div>
          </div>

          {/* Token Level */}
          <div className="flex items-center space-x-2">
            <Settings className="w-5 h-5 text-gray-500" />
            <div>
              <p className="font-medium text-gray-900">Access Level</p>
              <p className="text-sm text-gray-600">Level {consentRequest.token.payload.level}</p>
            </div>
          </div>

          {/* Expiration */}
          <div className="flex items-center space-x-2">
            <Clock className="w-5 h-5 text-gray-500" />
            <div>
              <p className="font-medium text-gray-900">Expires</p>
              <p className="text-sm text-gray-600">
                {expirationDate.toLocaleString()} 
                {customExpiration && (
                  <span className="text-blue-600 ml-2">(Custom)</span>
                )}
              </p>
            </div>
          </div>

          <Separator />

          {/* Capability Claims */}
          <div>
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-lg font-semibold">Requested Capabilities</h4>
              <div className="flex space-x-2">
                <Button variant="outline" size="sm" onClick={handleGrantAll}>
                  Grant All
                </Button>
                <Button variant="outline" size="sm" onClick={handleDenyAll}>
                  Deny All
                </Button>
              </div>
            </div>

            <div className="space-y-3">
              {consentRequest.token.payload.claims.map((claim, index) => (
                <CapabilityClaimItem
                  key={index}
                  claim={claim}
                  index={index}
                  isGranted={grantedClaims.has(index)}
                  onToggle={handleClaimToggle}
                />
              ))}
            </div>

            <div className="mt-4 p-3 bg-gray-50 rounded-lg">
              <p className="text-sm text-gray-600">
                <strong>Summary:</strong> {grantedClaims.size} of {consentRequest.token.payload.claims.length} capabilities granted
              </p>
            </div>
          </div>

          {/* Advanced Options */}
          <Dialog>
            <DialogTrigger asChild>
              <Button variant="outline" size="sm">
                Advanced Options
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Advanced Consent Options</DialogTitle>
                <DialogDescription>
                  Configure additional consent parameters and expiration settings.
                </DialogDescription>
              </DialogHeader>
              
              <div className="space-y-4">
                <div>
                  <Label htmlFor="custom-expiration">Custom Expiration</Label>
                  <input
                    id="custom-expiration"
                    type="datetime-local"
                    value={customExpiration ? new Date(customExpiration * 1000).toISOString().slice(0, 16) : ''}
                    onChange={(e) => {
                      if (e.target.value) {
                        setCustomExpiration(Math.floor(new Date(e.target.value).getTime() / 1000));
                      } else {
                        setCustomExpiration(undefined);
                      }
                    }}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    Leave empty to use default expiration
                  </p>
                </div>

                <div>
                  <Label htmlFor="notes">Notes (Optional)</Label>
                  <textarea
                    id="notes"
                    value={notes}
                    onChange={(e) => setNotes(e.target.value)}
                    placeholder="Add any notes about this consent decision..."
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                    rows={3}
                  />
                </div>
              </div>
            </DialogContent>
          </Dialog>

          {/* Action Buttons */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="flex items-center space-x-2">
              {isPartiallyGranted && (
                <Alert className="border-yellow-200 bg-yellow-50">
                  <AlertTriangle className="w-4 h-4 text-yellow-600" />
                  <AlertDescription className="text-yellow-800">
                    Partial consent selected. Some capabilities will be denied.
                  </AlertDescription>
                </Alert>
              )}
            </div>

            <div className="flex space-x-3">
              {onCancel && (
                <Button variant="outline" onClick={onCancel} disabled={disabled}>
                  Cancel
                </Button>
              )}
              <Button 
                variant="destructive" 
                onClick={handleDeny}
                disabled={disabled}
              >
                <XCircle className="w-4 h-4 mr-2" />
                Deny All
              </Button>
              <Button 
                onClick={handleGrant}
                disabled={!canGrant || disabled}
                className="bg-green-600 hover:bg-green-700"
              >
                <CheckCircle className="w-4 h-4 mr-2" />
                {isPartiallyGranted ? 'Grant Selected' : 'Grant All'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </TooltipProvider>
  );
};

export default ConsentCard;

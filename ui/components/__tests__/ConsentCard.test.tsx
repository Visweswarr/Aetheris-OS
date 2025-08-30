import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ConsentCard, ConsentRequest, ConsentDecision } from '../ConsentCard';

// Mock the UI components
jest.mock('../ui', () => ({
  Card: ({ children, className }: any) => <div data-testid="card" className={className}>{children}</div>,
  CardContent: ({ children }: any) => <div data-testid="card-content">{children}</div>,
  CardHeader: ({ children }: any) => <div data-testid="card-header">{children}</div>,
  CardTitle: ({ children }: any) => <h2 data-testid="card-title">{children}</h2>,
  CardDescription: ({ children }: any) => <p data-testid="card-description">{children}</p>,
  Button: ({ children, onClick, variant, disabled, className, size }: any) => (
    <button 
      data-testid={`button-${variant || 'default'}`} 
      onClick={onClick} 
      disabled={disabled}
      className={className}
      data-size={size}
    >
      {children}
    </button>
  ),
  Badge: ({ children, variant, className }: any) => (
    <span data-testid={`badge-${variant || 'default'}`} className={className}>
      {children}
    </span>
  ),
  Separator: () => <hr data-testid="separator" />,
  Alert: ({ children, className }: any) => (
    <div data-testid="alert" className={className}>{children}</div>
  ),
  AlertDescription: ({ children }: any) => (
    <p data-testid="alert-description">{children}</p>
  ),
  Checkbox: ({ id, checked, onCheckedChange }: any) => (
    <input
      type="checkbox"
      id={id}
      checked={checked}
      onChange={(e) => onCheckedChange(e.target.checked)}
      data-testid={`checkbox-${id}`}
    />
  ),
  Label: ({ children, htmlFor }: any) => (
    <label htmlFor={htmlFor} data-testid={`label-${htmlFor}`}>
      {children}
    </label>
  ),
  Dialog: ({ children }: any) => <div data-testid="dialog">{children}</div>,
  DialogContent: ({ children }: any) => <div data-testid="dialog-content">{children}</div>,
  DialogDescription: ({ children }: any) => <p data-testid="dialog-description">{children}</p>,
  DialogHeader: ({ children }: any) => <div data-testid="dialog-header">{children}</div>,
  DialogTitle: ({ children }: any) => <h3 data-testid="dialog-title">{children}</h3>,
  DialogTrigger: ({ children }: any) => <div data-testid="dialog-trigger">{children}</div>,
}));

// Mock lucide-react icons
jest.mock('lucide-react', () => ({
  Shield: () => <span data-testid="icon-shield">🛡️</span>,
  Clock: () => <span data-testid="icon-clock">⏰</span>,
  MapPin: () => <span data-testid="icon-map-pin">📍</span>,
  Smartphone: () => <span data-testid="icon-smartphone">📱</span>,
  Wifi: () => <span data-testid="icon-wifi">📶</span>,
  Database: () => <span data-testid="icon-database">💾</span>,
  FileText: () => <span data-testid="icon-file-text">📄</span>,
  Settings: () => <span data-testid="icon-settings">⚙️</span>,
  Info: () => <span data-testid="icon-info">ℹ️</span>,
  CheckCircle: () => <span data-testid="icon-check-circle">✅</span>,
  XCircle: () => <span data-testid="icon-x-circle">❌</span>,
  AlertTriangle: () => <span data-testid="icon-alert-triangle">⚠️</span>,
  Lock: () => <span data-testid="icon-lock">🔒</span>,
  User: () => <span data-testid="icon-user">👤</span>,
  Calendar: () => <span data-testid="icon-calendar">📅</span>,
  Globe: () => <span data-testid="icon-globe">🌍</span>,
  HardDrive: () => <span data-testid="icon-hard-drive">💿</span>,
  Network: () => <span data-testid="icon-network">🌐</span>,
  Eye: () => <span data-testid="icon-eye">👁️</span>,
  EyeOff: () => <span data-testid="icon-eye-off">👁️‍🗨️</span>,
}));

// Sample consent request data
const createSampleConsentRequest = (): ConsentRequest => ({
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
        {
          claim_type: "Scope",
          claim_data: {
            scope_name: "user_profile",
            scope_level: 2,
            scope_hierarchy: ["user", "profile"],
            scope_constraints: {},
          },
          constraints: {},
          metadata: {},
        },
        {
          claim_type: "Time",
          claim_data: {
            valid_from: Math.floor(Date.now() / 1000),
            valid_until: Math.floor(Date.now() / 1000) + 7200,
            time_constraints: {},
            recurring_patterns: [],
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
            device_type: "mobile",
            device_id: "device-123",
            device_capabilities: ["camera", "gps"],
            device_constraints: {},
          },
          constraints: {},
          metadata: {},
        },
        {
          claim_type: "Network",
          claim_data: {
            network_type: "wifi",
            network_id: "network-123",
            network_constraints: {},
            security_level: 2,
          },
          constraints: {},
          metadata: {},
        },
        {
          claim_type: "Data",
          claim_data: {
            data_type: "personal",
            data_classification: "confidential",
            access_level: 1,
            data_constraints: {},
          },
          constraints: {},
          metadata: {},
        },
        {
          claim_type: "Custom",
          claim_data: {
            claim_name: "analytics_consent",
            claim_value: true,
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
    logo: "https://example.com/logo.png",
  },
  context: {
    application: "User Profile Manager",
    action: "Update user profile information",
    description: "This app needs access to read and update your profile information to provide personalized services.",
    urgency: "medium",
  },
});

describe('ConsentCard', () => {
  let mockOnDecision: jest.Mock;
  let mockOnCancel: jest.Mock;
  let sampleRequest: ConsentRequest;

  beforeEach(() => {
    mockOnDecision = jest.fn();
    mockOnCancel = jest.fn();
    sampleRequest = createSampleConsentRequest();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Snapshot Tests', () => {
    it('renders correctly with all capability types', () => {
      const { container } = render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );
      
      expect(container).toMatchSnapshot();
    });

    it('renders correctly with expired token', () => {
      const expiredRequest = {
        ...sampleRequest,
        expires_at: Math.floor(Date.now() / 1000) - 3600, // 1 hour ago
      };

      const { container } = render(
        <ConsentCard
          consentRequest={expiredRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );
      
      expect(container).toMatchSnapshot();
    });

    it('renders correctly with different urgency levels', () => {
      const criticalRequest = {
        ...sampleRequest,
        context: { ...sampleRequest.context, urgency: 'critical' as const },
      };

      const { container } = render(
        <ConsentCard
          consentRequest={criticalRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );
      
      expect(container).toMatchSnapshot();
    });

    it('renders correctly with minimal claims', () => {
      const minimalRequest = {
        ...sampleRequest,
        token: {
          ...sampleRequest.token,
          payload: {
            ...sampleRequest.token.payload,
            claims: [sampleRequest.token.payload.claims[0]], // Only one claim
          },
        },
      };

      const { container } = render(
        <ConsentCard
          consentRequest={minimalRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );
      
      expect(container).toMatchSnapshot();
    });
  });

  describe('Rendering Tests', () => {
    it('renders the requester information correctly', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      expect(screen.getByText('Example App')).toBeInTheDocument();
      expect(screen.getByText('app.example.com')).toBeInTheDocument();
      expect(screen.getByText('Access user profile and preferences')).toBeInTheDocument();
    });

    it('renders all capability claims correctly', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Check that all claim types are rendered
      expect(screen.getByText(/file: user-profile\.json/)).toBeInTheDocument();
      expect(screen.getByText(/Action: update_profile/)).toBeInTheDocument();
      expect(screen.getByText(/Scope: user_profile \(Level 2\)/)).toBeInTheDocument();
      expect(screen.getByText(/Time: 2 hours/)).toBeInTheDocument();
      expect(screen.getByText(/Location: US/)).toBeInTheDocument();
      expect(screen.getByText(/Device: mobile/)).toBeInTheDocument();
      expect(screen.getByText(/Network: wifi/)).toBeInTheDocument();
      expect(screen.getByText(/Data: personal \(confidential\)/)).toBeInTheDocument();
      expect(screen.getByText(/Custom: analytics_consent/)).toBeInTheDocument();
    });

    it('renders urgency badge with correct styling', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const urgencyBadge = screen.getByText('medium');
      expect(urgencyBadge).toBeInTheDocument();
      expect(urgencyBadge.closest('[data-testid="badge-default"]')).toBeInTheDocument();
    });

    it('renders expiration information correctly', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const expirationText = screen.getByText(/Expires/);
      expect(expirationText).toBeInTheDocument();
    });
  });

  describe('Interaction Tests', () => {
    it('allows toggling individual claims', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Find the first checkbox and toggle it
      const firstCheckbox = screen.getByTestId('checkbox-claim-0');
      expect(firstCheckbox).toBeChecked(); // Should be checked by default

      await user.click(firstCheckbox);
      expect(firstCheckbox).not.toBeChecked();
    });

    it('allows granting all claims', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const grantAllButton = screen.getByTestId('button-outline');
      await user.click(grantAllButton);

      // All checkboxes should be checked
      sampleRequest.token.payload.claims.forEach((_, index) => {
        const checkbox = screen.getByTestId(`checkbox-claim-${index}`);
        expect(checkbox).toBeChecked();
      });
    });

    it('allows denying all claims', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const denyAllButton = screen.getByTestId('button-outline');
      await user.click(denyAllButton);

      // All checkboxes should be unchecked
      sampleRequest.token.payload.claims.forEach((_, index) => {
        const checkbox = screen.getByTestId(`checkbox-claim-${index}`);
        expect(checkbox).not.toBeChecked();
      });
    });

    it('shows claim details when expand button is clicked', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Find and click the first expand button
      const expandButtons = screen.getAllByTestId('button-ghost');
      const firstExpandButton = expandButtons[0];
      
      await user.click(firstExpandButton);

      // Should show detailed description
      expect(screen.getByText(/Access to file "user-profile\.json"/)).toBeInTheDocument();
    });

    it('calls onDecision with correct data when granting', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const grantButton = screen.getByTestId('button-default');
      await user.click(grantButton);

      expect(mockOnDecision).toHaveBeenCalledWith(
        expect.objectContaining({
          request_id: 'req-123',
          decision: 'granted',
          granted_claims: expect.arrayContaining(['Resource', 'Action', 'Scope', 'Time', 'Location', 'Device', 'Network', 'Data', 'Custom']),
          denied_claims: [],
        })
      );
    });

    it('calls onDecision with correct data when denying', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const denyButton = screen.getByTestId('button-destructive');
      await user.click(denyButton);

      expect(mockOnDecision).toHaveBeenCalledWith(
        expect.objectContaining({
          request_id: 'req-123',
          decision: 'denied',
          granted_claims: [],
          denied_claims: expect.arrayContaining(['Resource', 'Action', 'Scope', 'Time', 'Location', 'Device', 'Network', 'Data', 'Custom']),
        })
      );
    });

    it('calls onCancel when cancel button is clicked', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      const cancelButton = screen.getByTestId('button-outline');
      await user.click(cancelButton);

      expect(mockOnCancel).toHaveBeenCalled();
    });
  });

  describe('State Management Tests', () => {
    it('initializes all claims as granted by default', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // All checkboxes should be checked by default
      sampleRequest.token.payload.claims.forEach((_, index) => {
        const checkbox = screen.getByTestId(`checkbox-claim-${index}`);
        expect(checkbox).toBeChecked();
      });
    });

    it('tracks partial consent state correctly', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Uncheck the first claim
      const firstCheckbox = screen.getByTestId('checkbox-claim-0');
      await user.click(firstCheckbox);

      // Should show partial consent alert
      expect(screen.getByText(/Partial consent selected/)).toBeInTheDocument();
    });

    it('disables grant button when no claims are selected', async () => {
      const user = userEvent.setup();
      
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Deny all claims
      const denyAllButton = screen.getByTestId('button-outline');
      await user.click(denyAllButton);

      // Grant button should be disabled
      const grantButton = screen.getByTestId('button-default');
      expect(grantButton).toBeDisabled();
    });
  });

  describe('Edge Cases', () => {
    it('handles expired consent requests correctly', () => {
      const expiredRequest = {
        ...sampleRequest,
        expires_at: Math.floor(Date.now() / 1000) - 3600, // 1 hour ago
      };

      render(
        <ConsentCard
          consentRequest={expiredRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      expect(screen.getByText('Consent Request Expired')).toBeInTheDocument();
      expect(screen.getByText(/This consent request expired on/)).toBeInTheDocument();
    });

    it('handles requests without logo gracefully', () => {
      const requestWithoutLogo = {
        ...sampleRequest,
        requester: {
          ...sampleRequest.requester,
          logo: undefined,
        },
      };

      render(
        <ConsentCard
          consentRequest={requestWithoutLogo}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Should render without errors
      expect(screen.getByText('Example App')).toBeInTheDocument();
    });

    it('handles empty claims array', () => {
      const requestWithoutClaims = {
        ...sampleRequest,
        token: {
          ...sampleRequest.token,
          payload: {
            ...sampleRequest.token.payload,
            claims: [],
          },
        },
      };

      render(
        <ConsentCard
          consentRequest={requestWithoutClaims}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Should render without errors
      expect(screen.getByText('Example App')).toBeInTheDocument();
    });
  });

  describe('Accessibility Tests', () => {
    it('has proper labels for checkboxes', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      // Check that each checkbox has a proper label
      sampleRequest.token.payload.claims.forEach((_, index) => {
        const checkbox = screen.getByTestId(`checkbox-claim-${index}`);
        const label = screen.getByTestId(`label-claim-${index}`);
        
        expect(checkbox).toHaveAttribute('id', `claim-${index}`);
        expect(label).toHaveAttribute('htmlFor', `claim-${index}`);
      });
    });

    it('has proper button labels', () => {
      render(
        <ConsentCard
          consentRequest={sampleRequest}
          onDecision={mockOnDecision}
          onCancel={mockOnCancel}
        />
      );

      expect(screen.getByText('Grant All')).toBeInTheDocument();
      expect(screen.getByText('Deny All')).toBeInTheDocument();
      expect(screen.getByText('Deny All')).toBeInTheDocument();
      expect(screen.getByText('Grant All')).toBeInTheDocument();
    });
  });
});

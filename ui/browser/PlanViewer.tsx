import React from 'react';

export type PlanStep = {
  id: string;
  name: string;
  description?: string;
  dependencies: string[];
  approved?: boolean;
  status?: 'pending' | 'approved' | 'rejected' | 'completed';
};

export type Plan = {
  id: string;
  createdAt: string;
  steps: PlanStep[];
};

export type PlanViewerProps = {
  plan: Plan;
  onApproveStep?: (id: string) => void;
  onRejectStep?: (id: string) => void;
  onApproveAll?: () => void;
  onRejectAll?: () => void;
};

export const PlanViewer: React.FC<PlanViewerProps> = ({ plan, onApproveStep, onRejectStep, onApproveAll, onRejectAll }) => {
  return (
    <div style={{ fontFamily: 'sans-serif', padding: 12 }}>
      <h3>Plan Preview</h3>
      <div style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
        {onApproveAll && <button onClick={onApproveAll}>Approve All</button>}
        {onRejectAll && <button onClick={onRejectAll}>Reject All</button>}
      </div>
      <table style={{ borderCollapse: 'collapse', width: '100%' }}>
        <thead>
          <tr>
            <th style={th}>Step</th>
            <th style={th}>Depends On</th>
            <th style={th}>Status</th>
            <th style={th}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {plan.steps.map((s) => (
            <tr key={s.id}>
              <td style={td}>
                <div style={{ fontWeight: 600 }}>{sanitize(s.name)}</div>
                {s.description && <div style={{ color: '#555', fontSize: 12 }}>{sanitize(s.description)}</div>}
              </td>
              <td style={td}>{s.dependencies?.length ? s.dependencies.join(', ') : '—'}</td>
              <td style={td}>{s.status || (s.approved ? 'approved' : 'pending')}</td>
              <td style={td}>
                {onApproveStep && <button onClick={() => onApproveStep(s.id)} disabled={s.approved === true}>Approve</button>}
                {onRejectStep && <button onClick={() => onRejectStep(s.id)} style={{ marginLeft: 6 }} disabled={s.approved === false}>Reject</button>}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

const th: React.CSSProperties = { borderBottom: '1px solid #ccc', textAlign: 'left', padding: 6 };
const td: React.CSSProperties = { borderBottom: '1px solid #eee', padding: 6 };

function sanitize(s: string): string {
  // minimal sanitation to avoid accidental HTML injection in labels
  return s.replace(/[<>]/g, '_');
}

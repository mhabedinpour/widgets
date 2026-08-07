import React, { useState } from 'react';
import { Terminal, Send, CheckCircle, AlertTriangle } from 'lucide-react';

interface ApiInspectorProps {
  baseUrl?: string;
}

export const ApiInspector: React.FC<ApiInspectorProps> = ({ baseUrl = '' }) => {
  const [selectedEndpoint, setSelectedEndpoint] = useState<string>('GET /api/status');
  const [requestBody, setRequestBody] = useState<string>('{\n  "type": "clock",\n  "x": 0,\n  "y": 0,\n  "width": 64,\n  "height": 32,\n  "config": {\n    "utc_offset": "3600"\n  }\n}');
  const [responseOutput, setResponseOutput] = useState<string | null>(null);
  const [status, setStatus] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);

  const handleRunApi = async () => {
    setLoading(true);
    setResponseOutput(null);
    setStatus(null);

    const [method, path] = selectedEndpoint.split(' ');
    const url = `${baseUrl}${path}`;

    try {
      const options: RequestInit = {
        method,
        headers: { 'Content-Type': 'application/json' },
      };

      if (method === 'POST' || method === 'PUT') {
        options.body = requestBody;
      }

      const res = await fetch(url, options);
      setStatus(res.status);
      const text = await res.text();
      try {
        const json = JSON.parse(text);
        setResponseOutput(JSON.stringify(json, null, 2));
      } catch {
        setResponseOutput(text);
      }
    } catch (err: any) {
      setStatus(0);
      setResponseOutput(`Network Error / Connection Refused: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="glass-panel" style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
        <Terminal size={20} color="var(--accent-cyan)" />
        <h3 style={{ fontFamily: 'var(--font-heading)', fontSize: '1.1rem' }}>Picoserve REST API Tester</h3>
      </div>

      <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
        <select
          className="form-input"
          style={{ width: '220px', cursor: 'pointer' }}
          value={selectedEndpoint}
          onChange={(e) => setSelectedEndpoint(e.target.value)}
        >
          <option value="GET /api/status">GET /api/status</option>
          <option value="GET /api/widgets">GET /api/widgets</option>
          <option value="POST /api/widgets">POST /api/widgets</option>
          <option value="PUT /api/widgets/1">PUT /api/widgets/1</option>
          <option value="DELETE /api/widgets/1">DELETE /api/widgets/1</option>
          <option value="POST /api/upload/test.wasm">POST /api/upload/test.wasm</option>
          <option value="POST /api/reboot">POST /api/reboot</option>
        </select>

        <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.85rem', color: 'var(--text-muted)' }}>
          {selectedEndpoint.split(' ')[1]}
        </span>

        <button className="btn btn-primary" style={{ marginLeft: 'auto' }} onClick={handleRunApi} disabled={loading}>
          <Send size={16} />
          {loading ? 'Sending...' : 'Send Request'}
        </button>
      </div>

      {(selectedEndpoint.startsWith('POST') || selectedEndpoint.startsWith('PUT')) && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
          <label className="form-label">Request Payload (JSON)</label>
          <textarea
            className="form-input"
            rows={4}
            value={requestBody}
            onChange={(e) => setRequestBody(e.target.value)}
            style={{ width: '100%', resize: 'vertical' }}
          />
        </div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <label className="form-label">HTTP Response</label>
          {status !== null && (
            <span style={{
              fontSize: '0.75rem',
              fontFamily: 'var(--font-mono)',
              display: 'flex',
              alignItems: 'center',
              gap: '0.3rem',
              color: status === 200 ? 'var(--accent-emerald)' : '#ff4757'
            }}>
              {status === 200 ? <CheckCircle size={14} /> : <AlertTriangle size={14} />} Status Code: {status}
            </span>
          )}
        </div>
        <div className="code-block">
          {responseOutput || '// Click "Send Request" to test endpoint...'}
        </div>
      </div>
    </div>
  );
};

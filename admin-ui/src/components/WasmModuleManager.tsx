import React, { useState } from 'react';
import { Upload, FileCode, CheckCircle, AlertTriangle, RefreshCw } from 'lucide-react';

interface WasmModuleManagerProps {
  onUpload: (file: File) => Promise<void>;
}

export const WasmModuleManager: React.FC<WasmModuleManagerProps> = ({ onUpload }) => {
  const [file, setFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [status, setStatus] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      setFile(e.target.files[0]);
      setStatus(null);
    }
  };

  const handleUpload = async () => {
    if (!file) return;

    setUploading(true);
    setStatus(null);
    try {
      await onUpload(file);
      setStatus({ type: 'success', message: `Successfully uploaded ${file.name}` });
      setFile(null);
      // Clear file input
      const input = document.getElementById('wasm-upload-input') as HTMLInputElement;
      if (input) input.value = '';
    } catch (err: any) {
      setStatus({ type: 'error', message: `Upload failed: ${err.message}` });
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="glass-panel" style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
        <FileCode size={20} color="var(--accent-purple)" />
        <h3 style={{ fontFamily: 'var(--font-heading)', fontSize: '1.1rem' }}>WASM Module Manager</h3>
      </div>

      <p style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
        Upload compiled WebAssembly widgets to the device. These modules should be targetted for 
        <code>wasm32-unknown-unknown</code> and follow the LED Matrix OS widget API.
      </p>

      <div style={{ 
        border: '2px dashed var(--border-color)', 
        borderRadius: 'var(--radius-md)', 
        padding: '2rem', 
        display: 'flex', 
        flexDirection: 'column', 
        alignItems: 'center', 
        gap: '1rem',
        background: 'rgba(255, 255, 255, 0.02)'
      }}>
        <Upload size={32} color={file ? 'var(--accent-emerald)' : 'var(--text-dim)'} />
        
        <div style={{ textAlign: 'center' }}>
          {file ? (
            <div style={{ fontWeight: 600, color: 'var(--accent-emerald)' }}>{file.name}</div>
          ) : (
            <div style={{ color: 'var(--text-muted)' }}>Select a .wasm file to upload</div>
          )}
          <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
            {file ? `${(file.size / 1024).toFixed(1)} KB` : 'Maximum size: ~512 KB recommended'}
          </div>
        </div>

        <input
          id="wasm-upload-input"
          type="file"
          accept=".wasm"
          onChange={handleFileChange}
          style={{ display: 'none' }}
        />
        
        <div style={{ display: 'flex', gap: '0.75rem' }}>
          <button 
            className="btn btn-secondary" 
            onClick={() => document.getElementById('wasm-upload-input')?.click()}
            disabled={uploading}
          >
            Browse Files
          </button>
          <button 
            className="btn btn-primary" 
            onClick={handleUpload} 
            disabled={!file || uploading}
          >
            {uploading ? <RefreshCw className="spin" size={16} /> : <Upload size={16} />}
            {uploading ? 'Uploading...' : 'Start Upload'}
          </button>
        </div>
      </div>

      {status && (
        <div style={{ 
          padding: '0.75rem 1rem', 
          borderRadius: 'var(--radius-sm)', 
          fontSize: '0.85rem',
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
          background: status.type === 'success' ? 'rgba(0, 255, 136, 0.1)' : 'rgba(255, 71, 87, 0.1)',
          color: status.type === 'success' ? 'var(--accent-emerald)' : '#ff4757',
          border: `1px solid ${status.type === 'success' ? 'var(--accent-emerald)' : '#ff4757'}`
        }}>
          {status.type === 'success' ? <CheckCircle size={16} /> : <AlertTriangle size={16} />}
          {status.message}
        </div>
      )}

      <div style={{ marginTop: '0.5rem' }}>
        <h4 style={{ fontSize: '0.9rem', marginBottom: '0.5rem', color: 'var(--text-main)' }}>Usage Instructions</h4>
        <ul style={{ fontSize: '0.8rem', color: 'var(--text-muted)', paddingLeft: '1.25rem', display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
          <li>Binary files are stored in <code>/widgets/</code> on the internal flash.</li>
          <li>After uploading, go to <b>Widget Manager</b> and create a new widget with the same type name as your file (without .wasm).</li>
          <li>Existing modules with the same name will be overwritten.</li>
          <li>Reboot the device if the module does not appear to load correctly.</li>
        </ul>
      </div>
    </div>
  );
};

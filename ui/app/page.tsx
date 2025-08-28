"use client";
import { useState } from 'react';

export default function Page() {
  const [prompt, setPrompt] = useState("");
  const [output, setOutput] = useState("");
  const [loading, setLoading] = useState(false);

  async function send() {
    setLoading(true);
    try {
      const base = process.env.NEXT_PUBLIC_SERVER_URL || "http://localhost:8787";
      const res = await fetch(base + "/api/chat", {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt })
      });
      const json = await res.json();
      setOutput(json.output || "");
    } catch (e) {
      setOutput("error");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', height: '100vh' }}>
      <div style={{ padding: 16, borderRight: '1px solid #eee' }}>
        <h2>Chat</h2>
        <textarea value={prompt} onChange={e => setPrompt(e.target.value)} style={{ width: '100%', height: 200 }} />
        <div style={{ marginTop: 8 }}>
          <button onClick={send} disabled={loading}>
            {loading ? 'Sending…' : 'Send'}
          </button>
        </div>
      </div>
      <div style={{ padding: 16 }}>
        <h2>Output</h2>
        <pre style={{ whiteSpace: 'pre-wrap' }}>{output}</pre>
      </div>
    </div>
  );
}


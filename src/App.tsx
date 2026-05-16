import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [prompt, setPrompt] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const [finalAnswer, setFinalAnswer] = useState("");
  const [loading, setLoading] = useState(false);
  const [showLogs, setShowLogs] = useState(true);

  useEffect(() => {
    const unlistenPromise = listen<string>("agent-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  async function runAgent() {
    if (!prompt.trim()) return;

    setLogs([]);
    setFinalAnswer("");
    setLoading(true);

    try {
      const result = await invoke<string>("ask_ollama", {
        prompt,
      });

      setFinalAnswer(result);
    } catch (error) {
      setFinalAnswer(String(error));
    } finally {
      setLoading(false);
    }
  }

  return (
    <main>
      <h1>Minefia</h1>

      <textarea
        value={prompt}
        onChange={(e) => setPrompt(e.target.value)}
        placeholder="Ask Minefia..."
      />

      <br />

      <button onClick={runAgent} disabled={loading}>
        {loading ? "Running..." : "Run Agent"}
      </button>

      <button
        id="logsToggleBtn"
        onClick={() => setShowLogs((prev) => !prev)}
      >
        Logs {showLogs ? "▲" : "▼"}
      </button>

      {showLogs && (
        <div className="logs-container">
          {logs.length === 0 ? (
            <p className="empty-text">No logs yet.</p>
          ) : (
            logs.map((log, index) => (
              <pre key={index}>{log}</pre>
            ))
          )}
        </div>
      )}

      <h2>Final Answer</h2>
      <p>{finalAnswer}</p>
    </main>
  );
}

export default App;
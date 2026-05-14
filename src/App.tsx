import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface Task {
  id: string;
  title: string;
  status: "Queued" | "Running" | "Verifying" | "Completed" | "Failed";
}

interface SystemEvent {
  event_type: string;
  project_id: string;
  payload: any;
  timestamp: number;
}

function App() {
  const [projects, setProjects] = useState<string[]>([]);
  const [activeProjectId, setActiveProjectId] = useState<string | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [tasks, setTasks] = useState<Task[]>([
    { id: "1", title: "Architecture Planning", status: "Completed" },
    { id: "2", title: "Frontend Scaffold", status: "Running" },
    { id: "3", title: "Backend API Implementation", status: "Queued" },
    { id: "4", title: "Auth System Integration", status: "Queued" },
  ]);
  const [input, setInput] = useState("");

  useEffect(() => {
    // Initial fetch of projects
    invoke<string[]>("get_projects").then(setProjects);

    // Listen for backend events
    const unlisten = listen<SystemEvent>("asos-event", (event) => {
      setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${event.payload.event_type || 'INFO'}: ${JSON.stringify(event.payload)}`, ...prev].slice(0, 50));
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleSendMessage = () => {
    if (!input.trim()) return;
    // In a real scenario, this would call a Tauri command to inject a refinement
    console.log("Injecting refinement:", input);
    setInput("");
  };

  return (
    <div className="command-center">
      <aside className="sidebar">
        <div>
          <h2 style={{ marginBottom: "1rem", fontSize: "1.2rem" }}>ASOS Runtime</h2>
          <div className="text-muted" style={{ fontSize: "0.8rem" }}>v1.0.0-alpha</div>
        </div>

        <nav>
          <div className="text-muted" style={{ fontSize: "0.75rem", textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: "0.5rem" }}>Active Cells</div>
          {projects.length === 0 ? (
            <div className="text-muted" style={{ fontSize: "0.9rem" }}>No active projects</div>
          ) : (
            projects.map(p => (
              <div key={p} className={`card ${activeProjectId === p ? 'active' : ''}`} style={{ padding: '0.75rem', marginBottom: '0.5rem', cursor: 'pointer' }} onClick={() => setActiveProjectId(p)}>
                <span className="mono">{p.slice(0, 8)}...</span>
              </div>
            ))
          )}
        </nav>

        <div style={{ marginTop: "auto" }}>
          <button className="btn" style={{ width: "100%" }} onClick={() => invoke("create_project", { path: "./new-project" }).then(id => {
            setProjects(prev => [...prev, id as string]);
            setActiveProjectId(id as string);
          })}>
            New Project Cell
          </button>
        </div>
      </aside>

      <main className="main-view">
        <header style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div>
            <h1>Cognitive Dashboard</h1>
            <p className="text-muted">Supervising autonomous execution for <span className="mono">{activeProjectId || "None"}</span></p>
          </div>
          <div className="card" style={{ padding: "0.5rem 1rem", display: "flex", gap: "1rem", alignItems: "center" }}>
            <div className="status-indicator status-running"></div>
            <span style={{ fontWeight: 600 }}>Runtime Active</span>
          </div>
        </header>

        <section style={{ display: "grid", gridTemplateColumns: "1fr 320px", gap: "2rem" }}>
          <div className="card" style={{ minHeight: "400px" }}>
            <h3 style={{ marginBottom: "1.5rem" }}>Thought DAG Execution</h3>
            <div className="dag-visualization">
              {tasks.map(task => (
                <div key={task.id} className="dag-node">
                  <div className={`status-indicator status-${task.status.toLowerCase()}`}></div>
                  <div style={{ flex: 1 }}>
                    <div style={{ fontWeight: 500 }}>{task.title}</div>
                    <div className="text-muted" style={{ fontSize: "0.8rem" }}>ID: {task.id}</div>
                  </div>
                  <div className="mono" style={{ fontSize: "0.7rem", opacity: 0.6 }}>{task.status}</div>
                </div>
              ))}
            </div>
          </div>

          <div className="card">
            <h3 style={{ marginBottom: "1.5rem" }}>Agent Fabric</h3>
            <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
              <div className="dag-node" style={{ background: "rgba(99, 102, 241, 0.1)", border: "1px solid rgba(99, 102, 241, 0.2)" }}>
                <span style={{ fontWeight: 600 }}>PlannerAgent</span>
                <span className="text-muted" style={{ fontSize: "0.8rem" }}>Active</span>
              </div>
              <div className="dag-node" style={{ opacity: 0.5 }}>
                <span>CodingAgent</span>
                <span className="text-muted" style={{ fontSize: "0.8rem" }}>Idle</span>
              </div>
              <div className="dag-node" style={{ opacity: 0.5 }}>
                <span>VerifierAgent</span>
                <span className="text-muted" style={{ fontSize: "0.8rem" }}>Idle</span>
              </div>
            </div>
          </div>
        </section>

        <section className="card">
          <h3 style={{ marginBottom: "1rem" }}>Runtime Logs</h3>
          <div className="mono" style={{ color: "var(--text-muted)", height: "150px", overflowY: "auto", fontSize: "0.8rem" }}>
            {logs.length === 0 ? (
              <div className="text-muted">No logs recorded yet. Waiting for cognitive events...</div>
            ) : (
              logs.map((log, i) => <div key={i}>{log}</div>)
            )}
          </div>
        </section>
      </main>

      <div className="chat-bar-container">
        <div className="chat-bar">
          <input 
            type="text" 
            placeholder="Supervise the runtime... (e.g. 'prioritize auth', 'pause execution', 'add unit tests')" 
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSendMessage()}
          />
          <button className="btn" onClick={handleSendMessage}>Send Command</button>
        </div>
      </div>
    </div>
  );
}

export default App;

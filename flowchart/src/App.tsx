import { useCallback, useState, useRef } from 'react';
import type { Node, Edge, NodeChange, EdgeChange, Connection } from '@xyflow/react';
import {
  ReactFlow,
  useNodesState,
  useEdgesState,
  Controls,
  Background,
  BackgroundVariant,
  MarkerType,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge,
  Handle,
  Position,
  reconnectEdge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import './App.css';

const nodeWidth = 260;
const nodeHeight = 70;

type Phase = 'user' | 'command' | 'loop' | 'inner' | 'decision' | 'done';

const phaseColors: Record<Phase, { bg: string; border: string }> = {
  user: { bg: '#e8f4fd', border: '#2196f3' },
  command: { bg: '#f0f7ff', border: '#4a90d9' },
  loop: { bg: '#f5f5f5', border: '#666666' },
  inner: { bg: '#fff0f5', border: '#d63384' },
  decision: { bg: '#fff8e6', border: '#c9a227' },
  done: { bg: '#f0fff4', border: '#38a169' },
};

const allSteps: { id: string; label: string; description: string; phase: Phase }[] = [
  // User phase
  { id: '1', label: 'Create PRD with /prd', description: 'Describe your feature', phase: 'user' },
  { id: '2', label: 'Convert with /chief-wiggum:chief-wiggum', description: 'Creates prd.json', phase: 'user' },
  { id: '3', label: 'Run /chief-wiggum', description: 'Starts orchestration', phase: 'command' },
  // Command execution
  { id: '4', label: 'Executes chief-wiggum.sh', description: 'Shell script orchestrator', phase: 'command' },
  // Outer loop
  { id: '5', label: 'Picks next story', description: 'Finds passes: false', phase: 'loop' },
  { id: '6', label: 'Spawns Claude + /ralph-loop', description: 'Fresh context per story', phase: 'loop' },
  // Inner loop (ralph-loop)
  { id: '7', label: 'Implements story', description: 'Iterates until done', phase: 'inner' },
  { id: '8', label: 'Runs quality checks', description: 'typecheck, lint, test', phase: 'inner' },
  { id: '9', label: 'Commits changes', description: 'If checks pass', phase: 'inner' },
  { id: '10', label: 'Outputs STORY_COMPLETE', description: 'Signals completion', phase: 'inner' },
  // Back to outer loop
  { id: '11', label: 'Updates prd.json', description: 'Sets passes: true', phase: 'loop' },
  { id: '12', label: 'Logs to progress.txt', description: 'Saves learnings', phase: 'loop' },
  { id: '13', label: 'More stories?', description: '', phase: 'decision' },
  // Exit
  { id: '14', label: 'All stories complete!', description: 'PRD fully implemented', phase: 'done' },
];

const notes = [
  {
    id: 'note-1',
    appearsWithStep: 2,
    position: { x: 520, y: 80 },
    color: { bg: '#f5f0ff', border: '#8b5cf6' },
    content: `prd.json:
{
  "id": "US-001",
  "title": "Add priority field",
  "acceptanceCriteria": [...],
  "passes": false
}`,
  },
  {
    id: 'note-2',
    appearsWithStep: 6,
    position: { x: 520, y: 380 },
    color: { bg: '#fff0f5', border: '#d63384' },
    content: `claude --print "/ralph-loop
  \\"<story prompt>\\"
  --max-iterations 25
  --completion-promise STORY_COMPLETE"`,
  },
  {
    id: 'note-3',
    appearsWithStep: 12,
    position: { x: 520, y: 700 },
    color: { bg: '#fdf4f0', border: '#c97a50' },
    content: `Memory persists via:
• Git history
• progress.txt
• CLAUDE.md patterns`,
  },
];

function CustomNode({ data }: { data: { title: string; description: string; phase: Phase } }) {
  const colors = phaseColors[data.phase];
  return (
    <div
      className="custom-node"
      style={{
        backgroundColor: colors.bg,
        borderColor: colors.border
      }}
    >
      <Handle type="target" position={Position.Top} id="top" />
      <Handle type="target" position={Position.Left} id="left" />
      <Handle type="source" position={Position.Right} id="right" />
      <Handle type="source" position={Position.Bottom} id="bottom" />
      <Handle type="target" position={Position.Right} id="right-target" style={{ right: 0 }} />
      <Handle type="target" position={Position.Bottom} id="bottom-target" style={{ bottom: 0 }} />
      <Handle type="source" position={Position.Top} id="top-source" />
      <Handle type="source" position={Position.Left} id="left-source" />
      <div className="node-content">
        <div className="node-title">{data.title}</div>
        {data.description && <div className="node-description">{data.description}</div>}
      </div>
    </div>
  );
}

function NoteNode({ data }: { data: { content: string; color: { bg: string; border: string } } }) {
  return (
    <div
      className="note-node"
      style={{
        backgroundColor: data.color.bg,
        borderColor: data.color.border,
      }}
    >
      <pre>{data.content}</pre>
    </div>
  );
}

const nodeTypes = { custom: CustomNode, note: NoteNode };

const positions: { [key: string]: { x: number; y: number } } = {
  // User phase - vertical at top left
  '1': { x: 20, y: 20 },
  '2': { x: 60, y: 120 },
  '3': { x: 40, y: 220 },
  // Command execution
  '4': { x: 60, y: 320 },
  // Outer loop
  '5': { x: 40, y: 440 },
  '6': { x: 200, y: 540 },
  // Inner loop (ralph-loop) - right side
  '7': { x: 450, y: 440 },
  '8': { x: 480, y: 540 },
  '9': { x: 450, y: 640 },
  '10': { x: 250, y: 700 },
  // Back to outer loop
  '11': { x: 60, y: 640 },
  '12': { x: 40, y: 740 },
  '13': { x: 20, y: 850 },
  // Exit
  '14': { x: 280, y: 960 },
  // Notes
  ...Object.fromEntries(notes.map(n => [n.id, n.position])),
};

const edgeConnections: { source: string; target: string; sourceHandle?: string; targetHandle?: string; label?: string }[] = [
  // User phase
  { source: '1', target: '2', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '2', target: '3', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '3', target: '4', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '4', target: '5', sourceHandle: 'bottom', targetHandle: 'top' },
  // Outer loop starts
  { source: '5', target: '6', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '6', target: '7', sourceHandle: 'right', targetHandle: 'left' },
  // Inner loop (ralph-loop)
  { source: '7', target: '8', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '8', target: '9', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '9', target: '10', sourceHandle: 'left-source', targetHandle: 'right-target' },
  // Back to outer loop
  { source: '10', target: '11', sourceHandle: 'left-source', targetHandle: 'right-target' },
  { source: '11', target: '12', sourceHandle: 'bottom', targetHandle: 'top' },
  { source: '12', target: '13', sourceHandle: 'bottom', targetHandle: 'top' },
  // Decision
  { source: '13', target: '5', sourceHandle: 'top-source', targetHandle: 'left', label: 'Yes' },
  { source: '13', target: '14', sourceHandle: 'right', targetHandle: 'left', label: 'No' },
];

function createNode(step: typeof allSteps[0], visible: boolean, position?: { x: number; y: number }): Node {
  return {
    id: step.id,
    type: 'custom',
    position: position || positions[step.id],
    data: {
      title: step.label,
      description: step.description,
      phase: step.phase,
    },
    style: {
      width: nodeWidth,
      height: nodeHeight,
      opacity: visible ? 1 : 0,
      transition: 'opacity 0.5s ease-in-out',
      pointerEvents: visible ? 'auto' : 'none',
    },
  };
}

function createEdge(conn: typeof edgeConnections[0], visible: boolean): Edge {
  return {
    id: `e${conn.source}-${conn.target}`,
    source: conn.source,
    target: conn.target,
    sourceHandle: conn.sourceHandle,
    targetHandle: conn.targetHandle,
    label: visible ? conn.label : undefined,
    animated: visible,
    style: {
      stroke: '#222',
      strokeWidth: 2,
      opacity: visible ? 1 : 0,
      transition: 'opacity 0.5s ease-in-out',
    },
    labelStyle: {
      fill: '#222',
      fontWeight: 600,
      fontSize: 14,
    },
    labelShowBg: true,
    labelBgPadding: [8, 4] as [number, number],
    labelBgStyle: {
      fill: '#fff',
      stroke: '#222',
      strokeWidth: 1,
    },
    markerEnd: {
      type: MarkerType.ArrowClosed,
      color: '#222',
    },
  };
}

function createNoteNode(note: typeof notes[0], visible: boolean, position?: { x: number; y: number }): Node {
  return {
    id: note.id,
    type: 'note',
    position: position || positions[note.id],
    data: { content: note.content, color: note.color },
    style: {
      opacity: visible ? 1 : 0,
      transition: 'opacity 0.5s ease-in-out',
      pointerEvents: visible ? 'auto' : 'none',
    },
    draggable: true,
    selectable: false,
    connectable: false,
  };
}

function App() {
  const [visibleCount, setVisibleCount] = useState(1);
  const nodePositions = useRef<{ [key: string]: { x: number; y: number } }>({ ...positions });

  const getNodes = (count: number) => {
    const stepNodes = allSteps.map((step, index) =>
      createNode(step, index < count, nodePositions.current[step.id])
    );
    const noteNodes = notes.map(note => {
      const noteVisible = count >= note.appearsWithStep;
      return createNoteNode(note, noteVisible, nodePositions.current[note.id]);
    });
    return [...stepNodes, ...noteNodes];
  };

  const initialNodes = getNodes(1);
  const initialEdges = edgeConnections.map((conn, index) =>
    createEdge(conn, index < 0)
  );

  const [nodes, setNodes] = useNodesState(initialNodes);
  const [edges, setEdges] = useEdgesState(initialEdges);

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => {
      changes.forEach((change) => {
        if (change.type === 'position' && change.position) {
          nodePositions.current[change.id] = change.position;
        }
      });
      setNodes((nds) => applyNodeChanges(changes, nds));
    },
    [setNodes]
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      setEdges((eds) => applyEdgeChanges(changes, eds));
    },
    [setEdges]
  );

  const onConnect = useCallback(
    (connection: Connection) => {
      setEdges((eds) => addEdge({ ...connection, animated: true, style: { stroke: '#222', strokeWidth: 2 }, markerEnd: { type: MarkerType.ArrowClosed, color: '#222' } }, eds));
    },
    [setEdges]
  );

  const onReconnect = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      setEdges((eds) => reconnectEdge(oldEdge, newConnection, eds));
    },
    [setEdges]
  );

  const getEdgeVisibility = (conn: typeof edgeConnections[0], visibleStepCount: number) => {
    const sourceIndex = allSteps.findIndex(s => s.id === conn.source);
    const targetIndex = allSteps.findIndex(s => s.id === conn.target);
    return sourceIndex < visibleStepCount && targetIndex < visibleStepCount;
  };

  const handleNext = useCallback(() => {
    if (visibleCount < allSteps.length) {
      const newCount = visibleCount + 1;
      setVisibleCount(newCount);

      setNodes(getNodes(newCount));
      setEdges(
        edgeConnections.map((conn) =>
          createEdge(conn, getEdgeVisibility(conn, newCount))
        )
      );
    }
  }, [visibleCount, setNodes, setEdges]);

  const handlePrev = useCallback(() => {
    if (visibleCount > 1) {
      const newCount = visibleCount - 1;
      setVisibleCount(newCount);

      setNodes(getNodes(newCount));
      setEdges(
        edgeConnections.map((conn) =>
          createEdge(conn, getEdgeVisibility(conn, newCount))
        )
      );
    }
  }, [visibleCount, setNodes, setEdges]);

  const handleReset = useCallback(() => {
    setVisibleCount(1);
    nodePositions.current = { ...positions };
    setNodes(getNodes(1));
    setEdges(edgeConnections.map((conn, index) => createEdge(conn, index < 0)));
  }, [setNodes, setEdges]);

  return (
    <div className="app-container">
      <div className="header">
        <h1>Chief Wiggum: Autonomous PRD Executor</h1>
        <p>Two-tier architecture: /chief-wiggum (outer) + /ralph-loop (inner)</p>
      </div>
      <div className="legend">
        <span className="legend-item" style={{ backgroundColor: phaseColors.user.bg, borderColor: phaseColors.user.border }}>User</span>
        <span className="legend-item" style={{ backgroundColor: phaseColors.command.bg, borderColor: phaseColors.command.border }}>Command</span>
        <span className="legend-item" style={{ backgroundColor: phaseColors.loop.bg, borderColor: phaseColors.loop.border }}>Outer Loop</span>
        <span className="legend-item" style={{ backgroundColor: phaseColors.inner.bg, borderColor: phaseColors.inner.border }}>Inner Loop (ralph-loop)</span>
      </div>
      <div className="flow-container">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onReconnect={onReconnect}
          fitView
          fitViewOptions={{ padding: 0.2 }}
          nodesDraggable={true}
          nodesConnectable={true}
          edgesReconnectable={true}
          elementsSelectable={true}
          deleteKeyCode={['Backspace', 'Delete']}
          panOnDrag={true}
          panOnScroll={true}
          zoomOnScroll={true}
          zoomOnPinch={true}
          zoomOnDoubleClick={true}
          selectNodesOnDrag={false}
        >
          <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#ddd" />
          <Controls showInteractive={false} />
        </ReactFlow>
      </div>
      <div className="controls">
        <button onClick={handlePrev} disabled={visibleCount <= 1}>
          Previous
        </button>
        <span className="step-counter">
          Step {visibleCount} of {allSteps.length}
        </span>
        <button onClick={handleNext} disabled={visibleCount >= allSteps.length}>
          Next
        </button>
        <button onClick={handleReset} className="reset-btn">
          Reset
        </button>
      </div>
      <div className="instructions">
        Click Next to reveal each step • Forked from <a href="https://github.com/snarktank/ralph" target="_blank" rel="noopener noreferrer">snarktank/ralph</a>
      </div>
    </div>
  );
}

export default App;

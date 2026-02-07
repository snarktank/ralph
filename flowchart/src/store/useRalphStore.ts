import { create } from 'zustand';
import type { PRD, OrchestratorMessage, SubagentMessage, RalphStatus, PRDStatus } from '../types';

interface RalphStore {
  // PRD state
  prd: PRD | null;
  prdStatus: PRDStatus | null;

  // Ralph status
  ralphStatus: RalphStatus | null;

  // Messages
  orchestratorMessages: OrchestratorMessage[];
  subagentMessages: SubagentMessage[];

  // WebSocket
  ws: WebSocket | null;
  wsConnected: boolean;

  // Actions
  setPRD: (prd: PRD) => void;
  setPRDStatus: (status: PRDStatus) => void;
  setRalphStatus: (status: RalphStatus) => void;
  addOrchestratorMessage: (message: OrchestratorMessage) => void;
  addSubagentMessage: (message: SubagentMessage) => void;
  clearOrchestratorMessages: () => void;
  clearSubagentMessages: () => void;
  setWebSocket: (ws: WebSocket | null) => void;
  setWSConnected: (connected: boolean) => void;
}

export const useRalphStore = create<RalphStore>((set) => ({
  // Initial state
  prd: null,
  prdStatus: null,
  ralphStatus: null,
  orchestratorMessages: [],
  subagentMessages: [],
  ws: null,
  wsConnected: false,

  // Actions
  setPRD: (prd) => set({ prd }),
  setPRDStatus: (status) => set({ prdStatus: status }),
  setRalphStatus: (status) => set({ ralphStatus: status }),

  addOrchestratorMessage: (message) =>
    set((state) => ({
      orchestratorMessages: [...state.orchestratorMessages, message],
    })),

  addSubagentMessage: (message) =>
    set((state) => ({
      subagentMessages: [...state.subagentMessages, message],
    })),

  clearOrchestratorMessages: () => set({ orchestratorMessages: [] }),
  clearSubagentMessages: () => set({ subagentMessages: [] }),
  setWebSocket: (ws) => set({ ws }),
  setWSConnected: (connected) => set({ wsConnected: connected }),
}));

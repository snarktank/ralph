// API client for Ralph backend

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000';

export const api = {
  // PRD endpoints
  async getPRD() {
    const response = await fetch(`${API_BASE_URL}/api/prd/`);
    if (!response.ok) {
      if (response.status === 404) return null;
      throw new Error('Failed to fetch PRD');
    }
    return response.json();
  },

  async createPRD(data: { projectName: string; branchName: string; description: string }) {
    const response = await fetch(`${API_BASE_URL}/api/prd/`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error('Failed to create PRD');
    return response.json();
  },

  async updatePRD(data: any) {
    const response = await fetch(`${API_BASE_URL}/api/prd/`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error('Failed to update PRD');
    return response.json();
  },

  async deletePRD() {
    const response = await fetch(`${API_BASE_URL}/api/prd/`, {
      method: 'DELETE',
    });
    if (!response.ok) throw new Error('Failed to delete PRD');
    return response.json();
  },

  async addUserStory(story: any) {
    const response = await fetch(`${API_BASE_URL}/api/prd/stories`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(story),
    });
    if (!response.ok) throw new Error('Failed to add user story');
    return response.json();
  },

  async getPRDStatus() {
    const response = await fetch(`${API_BASE_URL}/api/prd/status`);
    if (!response.ok) throw new Error('Failed to fetch PRD status');
    return response.json();
  },

  // Ralph control endpoints
  async startRalph(maxIterations: number = 10, useCli: boolean = false) {
    const response = await fetch(`${API_BASE_URL}/api/ralph/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ max_iterations: maxIterations, use_cli: useCli }),
    });
    if (!response.ok) throw new Error('Failed to start Ralph');
    return response.json();
  },

  async stopRalph() {
    const response = await fetch(`${API_BASE_URL}/api/ralph/stop`, {
      method: 'POST',
    });
    if (!response.ok) throw new Error('Failed to stop Ralph');
    return response.json();
  },

  async getRalphStatus() {
    const response = await fetch(`${API_BASE_URL}/api/ralph/status`);
    if (!response.ok) throw new Error('Failed to fetch Ralph status');
    return response.json();
  },
};

export const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8000/ws';

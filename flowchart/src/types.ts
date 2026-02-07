// Type definitions for Ralph Web UI

export interface UserStory {
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: string[];
  priority: number;
  passes: boolean;
}

export interface PRD {
  projectName: string;
  branchName: string;
  description: string;
  userStories: UserStory[];
}

export interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: string;
  iteration?: number;
}

export interface OrchestratorMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

export interface SubagentMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  iteration?: number;
}

export interface RalphStatus {
  running: boolean;
  current_iteration: number;
  max_iterations: number;
}

export interface PRDStatus {
  exists: boolean;
  total_stories: number;
  completed_stories: number;
  incomplete_stories: number;
  all_complete: boolean;
}

// Full conversation history models
export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  tool_calls?: any[];
  tool_results?: any[];
}

export interface Conversation {
  id: string;
  type: 'orchestrator' | 'subagent';
  iteration?: number;
  story_id?: string;
  messages: Message[];
  created_at: string;
  updated_at: string;
}

export interface ConversationSummary {
  orchestrator: Conversation;
  subagents: Conversation[];
}

// Project models
export interface ProjectCreate {
  name: string;
  description: string;
  user_request: string;
}

export interface Project {
  id: string;
  name: string;
  description: string;
  path: string;
  port: number;
  stack: string;
  status: 'created' | 'installing' | 'running' | 'stopped' | 'error';
  created_at: string;
  url?: string;
  prd_path?: string;
  has_prd: boolean;
  has_ralph_config: boolean;
  ralph_status: 'not_started' | 'running' | 'stopped' | 'completed';
}

export interface PRDGenerateRequest {
  user_prompt: string;
}

export interface PRDUpdateRequest {
  update_prompt: string;
}

export interface PRDResponse {
  projectName: string;
  branchName: string;
  description: string;
  userStories: UserStory[];
  prd_path: string;
}

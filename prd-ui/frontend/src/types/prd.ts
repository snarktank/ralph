export interface Question {
  id: string;
  text: string;
  options: string[];
}

export interface UserStory {
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: string[];
  priority: number;
  passes: boolean;
  notes: string;
}

export interface PRDJSON {
  project: string;
  branchName: string;
  description: string;
  userStories: UserStory[];
}

export interface PRDFile {
  filename: string;
  name: string;
}

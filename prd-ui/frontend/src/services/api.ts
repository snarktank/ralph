import axios from 'axios';

const API_BASE_URL = '/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json'
  }
});

// Project endpoints
export const projectApi = {
  validate: (projectPath: string) =>
    api.post('/project/validate', { projectPath }),
  
  listPRDs: (projectPath: string) =>
    api.get('/project/prds', { params: { projectPath } })
};

// PRD endpoints
export const prdApi = {
  generateQuestions: (featureDescription: string) =>
    api.post('/prd/generate-questions', { featureDescription }),
  
  create: (data: {
    projectPath: string;
    featureName: string;
    answers?: any;
    prdContent?: string;
    projectName?: string;
  }) => api.post('/prd/create', data),
  
  read: (projectPath: string, featureName: string) =>
    api.get('/prd/read', { params: { projectPath, featureName } }),
  
  update: (data: {
    projectPath: string;
    featureName: string;
    content: string;
  }) => api.put('/prd/update', data)
};

// Convert endpoints
export const convertApi = {
  convert: (data: {
    projectPath?: string;
    prdPath?: string;
    prdContent?: string;
    projectName?: string;
  }) => api.post('/convert', data),
  
  save: (data: {
    projectPath: string;
    jsonData: any;
    projectName?: string;
  }) => api.post('/convert/save', data)
};

export default api;

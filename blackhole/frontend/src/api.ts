import axios from 'axios';
import type {
  Email,
  EmailListResponse,
  EmailDetailResponse,
  Inbox,
  Rule,
  User,
  AuthResponse,
  StatsOverview,
  LatencyStats,
  AuditLog,
  InternalNote,
  RuleConditions,
  RuleActions,
  EmailState,
} from './types';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

const api = axios.create({
  baseURL: API_URL,
  headers: { 'Content-Type': 'application/json' },
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

export const emailApi = {
  list: async (params?: {
    inbox_id?: string; state?: EmailState; assigned_to?: string;
    search?: string; page?: number; page_size?: number;
  }): Promise<EmailListResponse> => {
    const { data } = await api.get('/api/emails', { params });
    return data;
  },
  get: async (id: string): Promise<EmailDetailResponse> => {
    const { data } = await api.get(`/api/emails/${id}`);
    return data;
  },
  send: async (payload: {
    to: string[]; cc?: string[]; bcc?: string[];
    subject: string; body_text?: string; body_html?: string; reply_to?: string;
  }): Promise<Email> => {
    const { data } = await api.post('/api/emails', payload);
    return data;
  },
  update: async (id: string, updates: { state?: EmailState; assigned_to?: string | null; tags?: string[] }): Promise<Email> => {
    const { data } = await api.put(`/api/emails/${id}`, updates);
    return data;
  },
  updateState: async (id: string, state: EmailState): Promise<Email> => {
    const { data } = await api.put(`/api/emails/${id}/state`, { state });
    return data;
  },
  assign: async (id: string, user_id: string | null): Promise<Email> => {
    const { data } = await api.put(`/api/emails/${id}/assign`, { user_id });
    return data;
  },
  delete: async (id: string): Promise<void> => { await api.delete(`/api/emails/${id}`); },
  getNotes: async (id: string): Promise<InternalNote[]> => {
    const { data } = await api.get(`/api/emails/${id}/notes`);
    return data;
  },
  createNote: async (id: string, content: string): Promise<InternalNote> => {
    const { data } = await api.post(`/api/emails/${id}/notes`, { content });
    return data;
  },
  explainRouting: async (id: string): Promise<string> => {
    const { data } = await api.get(`/api/emails/${id}/routing`);
    return data;
  },
};

export const inboxApi = {
  list: async (): Promise<Inbox[]> => { const { data } = await api.get('/api/inboxes'); return data; },
  get: async (id: string): Promise<Inbox> => { const { data } = await api.get(`/api/inboxes/${id}`); return data; },
  create: async (inbox: { name: string; email_address: string; description?: string; is_shared: boolean; color?: string }): Promise<Inbox> => {
    const { data } = await api.post('/api/inboxes', inbox); return data;
  },
  update: async (id: string, updates: Partial<Inbox>): Promise<Inbox> => { const { data } = await api.put(`/api/inboxes/${id}`, updates); return data; },
  delete: async (id: string): Promise<void> => { await api.delete(`/api/inboxes/${id}`); },
  getEmails: async (id: string, params?: Record<string,unknown>): Promise<EmailListResponse> => {
    const { data } = await api.get(`/api/inboxes/${id}/emails`, { params }); return data;
  },
};

export const ruleApi = {
  list: async (): Promise<Rule[]> => { const { data } = await api.get('/api/rules'); return data; },
  get: async (id: string): Promise<Rule> => { const { data } = await api.get(`/api/rules/${id}`); return data; },
  create: async (rule: { name: string; description?: string; priority?: number; conditions: RuleConditions; actions: RuleActions }): Promise<Rule> => {
    const { data } = await api.post('/api/rules', rule); return data;
  },
  update: async (id: string, updates: Partial<Rule>): Promise<Rule> => { const { data } = await api.put(`/api/rules/${id}`, updates); return data; },
  delete: async (id: string): Promise<void> => { await api.delete(`/api/rules/${id}`); },
  toggle: async (id: string): Promise<Rule> => { const { data } = await api.put(`/api/rules/${id}/toggle`); return data; },
};

export const userApi = {
  list: async (): Promise<User[]> => { const { data } = await api.get('/api/users'); return data; },
  get: async (id: string): Promise<User> => { const { data } = await api.get(`/api/users/${id}`); return data; },
  create: async (user: { email: string; name: string; password: string; role: string }): Promise<User> => {
    const { data } = await api.post('/api/users', user); return data;
  },
  update: async (id: string, updates: Partial<User>): Promise<User> => { const { data } = await api.put(`/api/users/${id}`, updates); return data; },
};

export const authApi = {
  login: async (email: string, password: string): Promise<AuthResponse> => {
    const { data } = await api.post('/api/auth/login', { email, password });
    localStorage.setItem('auth_token', data.token);
    return data;
  },
  getCurrentUser: async (): Promise<User> => { const { data } = await api.get('/api/auth/me'); return data; },
  logout: () => { localStorage.removeItem('auth_token'); },
};

export const statsApi = {
  getOverview: async (): Promise<StatsOverview> => { const { data } = await api.get('/api/stats/overview'); return data; },
  getLatency: async (): Promise<LatencyStats> => { const { data } = await api.get('/api/stats/latency'); return data; },
};

export const auditApi = {
  list: async (params?: Record<string,unknown>): Promise<AuditLog[]> => { const { data } = await api.get('/api/audit', { params }); return data; },
};

export default api;

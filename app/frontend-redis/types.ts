

export interface User {
  id: string;
  email: string;
  username: string;
  first_name: string;
  last_name: string;
}

export interface AuthData {
  token: string;
  user: User;
}

export interface Organization {
  id: string;
  name: string;
  slug: string;
  description: string;
  owner_id: string;
  is_active: boolean;
  plan: string;
  max_redis_instances: number;
  max_api_keys: number;
  created_at: string;
  updated_at: string;
}

export interface RedisInstance {
  id: string;
  name: string;
  slug: string;
  domain: string;
  status: 'running' | 'pending' | 'stopped' | 'error';
  max_memory: number;
  redis_version: string;
  created_at: string;
}

export interface ApiKey {
  id: string;
  key_name: string;
  organization_id: string;
  created_at: string;
  last_used_at: string | null;
}

export interface PaginatedResponse<T> {
  items: T[];
  total_count: number;
  page: number;
  limit: number;
  total_pages: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message: string | null;
  timestamp: string;
}

export interface HealthStatus {
  database: 'healthy' | 'unhealthy' | string;
  status: 'ok' | 'error' | string;
  timestamp: string;
}

export interface SystemStats {
  tables: {
    api_keys: number;
    organizations: number;
    redis_instances: number;
    users: number;
  };
  timestamp: string;
}

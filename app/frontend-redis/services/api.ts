import { 
  ApiResponse, 
  AuthData, 
  Organization, 
  PaginatedResponse, 
  RedisInstance, 
  User,
  ApiKey,
  HealthStatus,
  SystemStats
} from '../types';

const BASE_URL = 'http://localhost:8080';

async function request<T>(endpoint: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
  const url = `${BASE_URL}${endpoint}`;
  
  const headers = {
    'Content-Type': 'application/json',
    ...options.headers,
  };

  const response = await fetch(url, { ...options, headers });

  const result = await response.json();

  if (!response.ok) {
    throw new Error(result.message || `HTTP error! status: ${response.status}`);
  }
  
  if(!result.success) {
    throw new Error(result.message || 'API returned an error');
  }
  return result;
}

export const register = async (userData: Omit<User, 'id'> & { password: string }) => {
  const response = await request<User>('/auth/register', {
    method: 'POST',
    body: JSON.stringify(userData),
  });
  return response.data;
};

export const login = async (credentials: Pick<User, 'email'> & { password: string }) => {
  const response = await request<AuthData>('/auth/login', {
    method: 'POST',
    body: JSON.stringify(credentials),
  });
  return response.data;
};

export const getOrganizations = async (token: string, page = 1, limit = 20) => {
  const response = await request<PaginatedResponse<Organization>>(`/api/organizations?page=${page}&limit=${limit}`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  return response.data;
};

export const createOrganization = async (token: string, orgData: { name: string; description: string; slug: string }) => {
    const response = await request<Organization>('/api/organizations', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` },
        body: JSON.stringify(orgData)
    });
    return response.data;
};

export const updateOrganization = async (token: string, orgId: string, orgData: { name: string; description: string; slug: string }) => {
    const response = await request<Organization>(`/api/organizations/${orgId}`, {
        method: 'PUT',
        headers: { 'Authorization': `Bearer ${token}` },
        body: JSON.stringify(orgData)
    });
    return response.data;
};

export const deleteOrganization = async (token: string, orgId: string) => {
    const response = await request<null>(`/api/organizations/${orgId}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${token}` }
    });
    return response;
};

export const getRedisInstances = async (token: string, orgId: string) => {
  const response = await request<{ items: RedisInstance[] }>(`/api/organizations/${orgId}/redis-instances`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  return response.data;
};

export const getApiKeys = async (token: string, orgId: string) => {
  const response = await request<{ items: ApiKey[] }>(`/api/organizations/${orgId}/api-keys`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  return response.data;
};

export const createRedisInstance = async (token: string, orgId: string, instanceData: any) => {
  const response = await request<RedisInstance>(`/api/organizations/${orgId}/redis-instances`, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${token}` },
    body: JSON.stringify(instanceData),
  });
  return response.data;
};

export const deleteRedisInstance = async (token: string, orgId: string, instanceId: string) => {
    const response = await request<null>(`/api/organizations/${orgId}/redis-instances/${instanceId}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${token}` }
    });
    return response;
};

export const getSystemHealth = async (): Promise<HealthStatus> => {
  const url = `${BASE_URL}/health`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Health check failed! status: ${response.status}`);
  }
  const result: HealthStatus = await response.json();
  return result;
};

export const getSystemStats = async (): Promise<SystemStats> => {
  const url = `${BASE_URL}/stats`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Stats fetch failed! status: ${response.status}`);
  }
  const result: SystemStats = await response.json();
  return result;
};

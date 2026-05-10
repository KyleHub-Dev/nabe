export type EdgeNodeStatus = 'enrolling' | 'online' | 'offline' | 'revoked';

export interface HeartbeatRequest {
  edgeNodeId: string;
  status: EdgeNodeStatus;
  observedAt: string;
  engineInventory?: Array<{ kind: string; version?: string }>;
}

export interface HeartbeatResponse {
  accepted: boolean;
  jobs: Job[];
}

export interface EnrollmentRequest {
  token: string;
  nodeName: string;
  agentVersion: string;
}

export interface EnrollmentResponse {
  accepted: boolean;
  edgeNodeId?: string;
  apiBaseUrl?: string;
}

export interface Job {
  id: string;
  type: string;
  payload: Record<string, unknown>;
}

export interface JobResult {
  jobId: string;
  success: boolean;
  message?: string;
  payload?: Record<string, unknown>;
}

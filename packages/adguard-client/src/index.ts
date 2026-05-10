export interface AdGuardClientConfig {
  baseUrl: string;
  username?: string;
  password?: string;
}

export interface DeviceClientInput {
  name: string;
  clientId: string;
  tags?: string[];
}

export class AdGuardClient {
  constructor(private readonly config: AdGuardClientConfig) {}

  async addClient(input: DeviceClientInput): Promise<void> {
    void input;
    this.notImplemented();
  }

  async removeClient(clientId: string): Promise<void> {
    void clientId;
    this.notImplemented();
  }

  async listClients(): Promise<DeviceClientInput[]> {
    this.notImplemented();
  }

  async getQueryLog(clientId?: string): Promise<unknown[]> {
    void clientId;
    this.notImplemented();
  }

  async getStats(clientId?: string): Promise<Record<string, unknown>> {
    void clientId;
    this.notImplemented();
  }

  private notImplemented(): never {
    throw new Error(`AdGuard client placeholder for ${this.config.baseUrl}`);
  }
}

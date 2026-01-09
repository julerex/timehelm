import type { Activity, PlayerData, Position } from '../entities/Player';

export type WebSocketMessage =
    | { type: 'Join'; player: PlayerData }
    | { type: 'Leave'; player_id: string }
    | { type: 'Move'; player_id: string; position: Position; rotation: number; is_moving: boolean }
    | { type: 'SetActivity'; player_id: string; activity: Activity }
    | { type: 'ActivityChanged'; player_id: string; activity: Activity }
    | { type: 'WorldState'; players: PlayerData[] }
    | { type: 'TimeSync'; game_time_minutes: number };

export interface NetworkEventHandlers {
    onWorldState: (players: PlayerData[]) => void;
    onPlayerJoin: (player: PlayerData) => void;
    onPlayerLeave: (playerId: string) => void;
    onPlayerMove: (playerId: string, position: Position, rotation: number, isMoving: boolean) => void;
    onActivityChanged: (playerId: string, activity: Activity) => void;
    onTimeSync: (gameTimeMinutes: number) => void;
}

export class NetworkManager {
    private ws: WebSocket | null = null;
    private handlers: NetworkEventHandlers;
    private reconnectTimeout: number = 3000;

    constructor(handlers: NetworkEventHandlers) {
        this.handlers = handlers;
    }

    // --- Public Methods ---

    public connect(): void {
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${wsProtocol}//${window.location.host}/ws`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = this.handleOpen;
        this.ws.onmessage = this.handleMessage;
        this.ws.onerror = this.handleError;
        this.ws.onclose = this.handleClose;
    }

    public disconnect(): void {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }

    public sendJoin(player: PlayerData): void {
        this.sendMessage({ type: 'Join', player });
    }

    public sendMove(playerId: string, position: Position, rotation: number, isMoving: boolean): void {
        this.sendMessage({
            type: 'Move',
            player_id: playerId,
            position,
            rotation,
            is_moving: isMoving
        });
    }

    public sendSetActivity(playerId: string, activity: Activity): void {
        this.sendMessage({
            type: 'SetActivity',
            player_id: playerId,
            activity
        });
    }

    public isConnected(): boolean {
        return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
    }

    // --- Private Methods ---

    private sendMessage(message: WebSocketMessage): void {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    private handleOpen = (): void => {
        console.log('WebSocket connected');
    };

    private handleMessage = (event: MessageEvent): void => {
        try {
            const message = JSON.parse(event.data) as WebSocketMessage;
            this.processMessage(message);
        } catch (error) {
            console.error('Failed to parse message:', error);
        }
    };

    private handleError = (error: Event): void => {
        console.error('WebSocket error:', error);
    };

    private handleClose = (): void => {
        console.log('WebSocket disconnected');
        setTimeout(() => this.connect(), this.reconnectTimeout);
    };

    private processMessage(message: WebSocketMessage): void {
        switch (message.type) {
            case 'WorldState':
                this.handlers.onWorldState(message.players);
                break;
            case 'Join':
                this.handlers.onPlayerJoin(message.player);
                break;
            case 'Leave':
                this.handlers.onPlayerLeave(message.player_id);
                break;
            case 'Move':
                this.handlers.onPlayerMove(
                    message.player_id,
                    message.position,
                    message.rotation,
                    message.is_moving
                );
                break;
            case 'ActivityChanged':
                this.handlers.onActivityChanged(message.player_id, message.activity);
                break;
            case 'TimeSync':
                this.handlers.onTimeSync(message.game_time_minutes);
                break;
        }
    }
}

/**
 * Block system types for P2P interactive terminal
 */

export type BlockType = 'text' | 'code' | 'file' | 'widget' | 'system';

export interface BaseBlock {
  id: string;
  type: BlockType;
  senderId: string;
  senderName?: string;
  timestamp: number;
  isLocal: boolean;
}

export interface TextBlock extends BaseBlock {
  type: 'text';
  content: string;
  format?: 'plain' | 'markdown';
}

export interface CodeBlock extends BaseBlock {
  type: 'code';
  content: string;
  language: string;
  filename?: string;
  showLineNumbers?: boolean;
}

export interface FileBlock extends BaseBlock {
  type: 'file';
  filename: string;
  size: number;
  mimeType: string;
  previewUrl?: string;
  transferProgress?: number;
  transferStatus: 'pending' | 'transferring' | 'completed' | 'failed';
  data?: ArrayBuffer;
}

export interface WidgetBlock extends BaseBlock {
  type: 'widget';
  widgetType: WidgetType;
  config: WidgetConfig;
  responses?: WidgetResponse[];
}

export interface SystemBlock extends BaseBlock {
  type: 'system';
  message: string;
  level: 'info' | 'warning' | 'error' | 'success';
}

export type Block = TextBlock | CodeBlock | FileBlock | WidgetBlock | SystemBlock;

// Widget types
export type WidgetType = 'button' | 'input' | 'poll' | 'progress' | 'confirm';

export interface ButtonWidgetConfig {
  type: 'button';
  label: string;
  variant?: 'primary' | 'secondary' | 'danger';
  action: string;
}

export interface InputWidgetConfig {
  type: 'input';
  label: string;
  placeholder?: string;
  inputType?: 'text' | 'number' | 'password';
}

export interface PollWidgetConfig {
  type: 'poll';
  question: string;
  options: string[];
  allowMultiple?: boolean;
}

export interface ProgressWidgetConfig {
  type: 'progress';
  label: string;
  value: number;
  max: number;
}

export interface ConfirmWidgetConfig {
  type: 'confirm';
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
}

export type WidgetConfig = 
  | ButtonWidgetConfig 
  | InputWidgetConfig 
  | PollWidgetConfig 
  | ProgressWidgetConfig
  | ConfirmWidgetConfig;

export interface WidgetResponse {
  responderId: string;
  responderName?: string;
  timestamp: number;
  value: unknown;
}

// Block composer state
export interface ComposerState {
  mode: 'text' | 'code' | 'widget';
  content: string;
  language?: string;
  widgetType?: WidgetType;
  widgetConfig?: Partial<WidgetConfig>;
}

// P2P Message envelope
export interface P2PMessage {
  id: string;
  type: 'block' | 'typing' | 'ack' | 'widget-response';
  payload: Block | TypingIndicator | MessageAck | WidgetResponsePayload;
  timestamp: number;
}

export interface TypingIndicator {
  peerId: string;
  peerName?: string;
  isTyping: boolean;
}

export interface MessageAck {
  messageId: string;
  status: 'delivered' | 'read';
}

export interface WidgetResponsePayload {
  blockId: string;
  response: WidgetResponse;
}

// Block registry
export interface BlockRenderer {
  type: BlockType;
  component: string;
  version: string;
}

export interface BlockRegistry {
  renderers: Map<BlockType, BlockRenderer>;
  register(renderer: BlockRenderer): void;
  get(type: BlockType): BlockRenderer | undefined;
}

/**
 * Mock entry point - loads mocks before app initialization
 */

import { installMocks } from './tauri';

// Install mocks immediately
installMocks();

// Then load the real app
import('../main');

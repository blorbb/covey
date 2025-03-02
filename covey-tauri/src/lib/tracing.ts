/**
 * Utilities for logging to Covey logger.
 */

import { invoke } from "@tauri-apps/api/core";

export default {
  error: (log: string) => void invoke("log_error", { log }),
  warn: (log: string) => void invoke("log_warn", { log }),
  info: (log: string) => void invoke("log_info", { log }),
  debug: (log: string) => void invoke("log_debug", { log }),
  trace: (log: string) => void invoke("log_trace", { log }),
};

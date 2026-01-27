import "unplugin-icons/types/svelte";

declare global {
  interface Window {
    showSaveFilePicker?: (options?: {
      suggestedName?: string;
      types?: Array<{
        description?: string;
        accept?: Record<string, string[]>;
      }>;
    }) => Promise<{
      createWritable: () => Promise<{
        write: (data: Blob) => Promise<void>;
        close: () => Promise<void>;
      }>;
    }>;
    __TAURI_INTERNALS__: {
      invoke: typeof invoke;
      transformCallback: typeof transformCallback;
      unregisterCallback: (id: number) => void;
      runCallback: (id: number, data: unknown) => void;
      callbacks: Map<number, (data: unknown) => void>;
      convertFileSrc: typeof convertFileSrc;
      ipc: (message: {
        cmd: string;
        callback: number;
        error: number;
        payload: unknown;
        options?: InvokeOptions;
      }) => void;
      metadata: {
        currentWindow: WindowDef;
        currentWebview: WebviewDef;
      };
      plugins: {
        path: {
          sep: string;
          delimiter: string;
        };
      };
    };
    __TAURI_EVENT_PLUGIN_INTERNALS__: {
      unregisterListener: (event: string, eventId: number) => void;
    };
  }
}

export { };

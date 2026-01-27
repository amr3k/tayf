import 'unplugin-icons/types/svelte'

declare global {
  interface Window {
    showSaveFilePicker?: (options?: {
      suggestedName?: string
      types?: Array<{
        description?: string
        accept?: Record<string, string[]>
      }>
    }) => Promise<{
      createWritable: () => Promise<{
        write: (data: Blob) => Promise<void>
        close: () => Promise<void>
      }>
    }>
  }
}

export {}


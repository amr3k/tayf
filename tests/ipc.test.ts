import { invoke } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

describe("Tauri IPC Mocking Examples", () => {
  beforeEach(() => {
    mockIPC((cmd, args) => {
      if (cmd === "get_config") {
        return { theme: "system", lang: "en", canvas_background_color: "#FFFFFF" };
      }
      if (cmd === "set_config") {
        return undefined;
      }
      if (cmd === "read_file_content") {
        return new Uint8Array([123, 125, 10, 32, 34, 118, 34, 58, 32, 53, 44, 125]);
      }
    });
  });

  afterEach(() => {
    clearMocks();
  });

  it("should mock get_config command", async () => {
    const config = await invoke("get_config");
    expect(config).toEqual({
      theme: "system",
      lang: "en",
      canvas_background_color: "#FFFFFF",
    });
  });

  it("should mock set_config command", async () => {
    const result = await invoke("set_config", {
      config: { theme: "dark", lang: "en" },
    });
    expect(result).toBeUndefined();
  });

  it("should mock read_file_content command", async () => {
    const content = await invoke<Uint8Array>("read_file_content", {
      filePath: "/path/to/file.json",
    });
    expect(content).toBeInstanceOf(Uint8Array);
    expect(content.length).toBeGreaterThan(0);
  });
});

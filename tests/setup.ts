import "@testing-library/jest-dom";
import * as matchers from "@testing-library/jest-dom/matchers";
import { randomFillSync } from "node:crypto";
import { expect } from "vitest";

Object.keys(matchers).forEach((key) => {
  expect.extend({ [key]: matchers[key] });
});

Object.defineProperty(global, "crypto", {
  value: {
    getRandomValues: (buffer: Uint8Array) => randomFillSync(buffer),
  },
});

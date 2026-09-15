import { describe, expect, it } from "vitest";

import App from "./App";

describe("App composition contract", () => {
  it("exports the thin application composition root", () => {
    expect(typeof App).toBe("function");
  });
});

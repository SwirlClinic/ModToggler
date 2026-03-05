import { vi } from "vitest";
import "@testing-library/jest-dom/vitest";

// Mock @tauri-apps/api/core so components can be tested without a real Tauri context
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

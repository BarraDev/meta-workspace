import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { spawnSync } from "node:child_process";

interface MwDecision {
  decision?: "allow" | "deny" | "warn" | "modify";
  reason?: string;
  message?: string;
  input?: unknown;
}

function claudeShapedEvent(event: { toolName: string; input: unknown }) {
  return {
    hook_event_name: "PreToolUse",
    tool_name: event.toolName,
    tool_input: event.input,
  };
}

function checkPolicy(payload: unknown): MwDecision | undefined {
  const result = spawnSync("mw", ["policy", "check"], {
    input: JSON.stringify(payload),
    encoding: "utf8",
  });

  const stdout = result.stdout?.trim();
  if (!stdout) {
    if (result.status && result.status !== 0) {
      return { decision: "deny", reason: result.stderr?.trim() || "mw policy check failed" };
    }
    return undefined;
  }

  try {
    return JSON.parse(stdout) as MwDecision;
  } catch {
    return { decision: "deny", reason: `mw policy check returned invalid JSON: ${stdout}` };
  }
}

export default function (pi: ExtensionAPI) {
  pi.on("tool_call", async (event) => {
    const decision = checkPolicy(claudeShapedEvent(event));

    if (decision?.decision === "deny") {
      return {
        block: true,
        reason: decision.reason || "Blocked by mw policy",
      };
    }

    if (decision?.decision === "modify" && decision.input && typeof decision.input === "object") {
      Object.assign(event.input as object, decision.input);
    }
  });
}

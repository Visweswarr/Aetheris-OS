// AICore TypeScript Bridge - minimal CLI wrapper
import { spawnSync } from 'node:child_process';

export interface PlanResult {
  plan?: any;
  success: boolean;
  generation_time_us: number;
  error?: string;
}

export function submitGoal(goal: string, binary = 'aicore'): PlanResult {
  const res = spawnSync(binary, ['submit', goal], { encoding: 'utf-8' });
  if (res.status !== 0) {
    throw new Error(`aicore CLI failed: ${res.stderr || res.stdout}`);
  }
  try {
    return JSON.parse(res.stdout) as PlanResult;
  } catch (e) {
    throw new Error(`Invalid JSON from aicore CLI: ${e}`);
  }
}

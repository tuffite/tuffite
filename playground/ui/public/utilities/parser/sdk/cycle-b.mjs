import { cycleA } from './cycle-a.mjs';
export function cycleB(depth) { return depth === 0 ? 'cycle' : cycleA(depth - 1); }

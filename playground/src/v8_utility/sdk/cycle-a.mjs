import { cycleB } from './cycle-b.mjs';
export function cycleA(depth) { return depth === 0 ? 'cycle' : cycleB(depth - 1); }
